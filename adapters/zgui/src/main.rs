use anyhow::{Context, Result};
use bench_core::{
    AdapterEvent, FrameSample, Framework, FrameworkMemory, MeasurementMode, PipelineSample,
    SamplePhase, Scenario, StyleComplexity, Topology, Workload,
};
use clap::{Parser, ValueEnum};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::io::BufRead;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use zgui::app::Fonts;
use zgui::prelude::*;
use zgui::reactive::RwSignal;
use zgui::view::{AnyView, ClassName};
use zgui_platform::{
    AppHandler, IdlePolicy, PlatformCx, Surface, SurfaceEvent, SurfaceId, WakeReason,
};
use zgui_runtime::{FrameProbe, Window};

const FONT: &[u8] = include_bytes!("../../../assets/IBMPlexSans-Regular.ttf");
const SHEET: &str = zgui::css!(
    r#"
    :root { width: 100%; height: 100%; overflow: hidden; padding: 8px;
            background-color: #0b1020; color: #e8eefc; font-family: "IBM Plex Sans"; font-size: 12px }
    .root { width: 100%; height: 100%; flex-direction: row; flex-wrap: wrap; gap: 4px; overflow: hidden }
    .item { min-width: 18px; min-height: 16px; padding: 1px 4px; flex-direction: column;
            background-color: #18243a; color: #e8eefc }
    .item.changed { background-color: #357d74 }
    .item.layout-changed { min-width: 38px; padding-left: 12px }
    .item.decorated { border: 1px solid #3b4d6b; border-radius: 4px;
                      box-shadow: 0 2px 4px rgba(0,0,0,0.35) }
    .nested { flex-direction: column; gap: 2px; padding-left: 2px }
"#
);

#[derive(Parser)]
struct Args {
    #[arg(long)]
    scenario: String,
    #[arg(long, value_enum)]
    mode: ModeArg,
}
#[derive(Clone, Copy, ValueEnum)]
enum ModeArg {
    Headline,
    Diagnostic,
}
impl From<ModeArg> for MeasurementMode {
    fn from(v: ModeArg) -> Self {
        match v {
            ModeArg::Headline => Self::Headline,
            ModeArg::Diagnostic => Self::Diagnostic,
        }
    }
}

type Stimulus = Rc<RefCell<Option<Box<dyn Fn(u64)>>>>;

type LocalSignal<T> = RwSignal<T, LocalStorage>;

fn item(spec: &Scenario, id: usize, tick: LocalSignal<u64>, children: Vec<AnyView>) -> AnyView {
    let paint = matches!(
        spec.workload,
        Workload::PaintUpdate | Workload::InputLatency
    );
    let layout = matches!(spec.workload, Workload::LayoutUpdate);
    let text_update = matches!(spec.workload, Workload::TextUpdate);
    let decorated = spec.style == StyleComplexity::Decorated;
    let element = zgui::elements::column()
        .class("item")
        .class_toggle(ClassName::new("changed"), move || {
            paint && tick.get() % 2 == 1
        })
        .class_toggle(ClassName::new("layout-changed"), move || {
            layout && tick.get() % 2 == 1
        })
        .class_toggle(ClassName::new("decorated"), decorated)
        .child(zgui::elements::text().child(move || {
            if text_update && tick.get() > 0 {
                format!("item {id:05} · {:04}", tick.get())
            } else {
                format!("item {id:05}")
            }
        }))
        .children(children);
    AnyView::new(element)
}

fn balanced(
    spec: &Scenario,
    id: usize,
    children: &[Vec<usize>],
    ticks: &[LocalSignal<u64>],
) -> AnyView {
    let nested = children[id]
        .iter()
        .map(|&child| balanced(spec, child, children, ticks))
        .collect();
    item(spec, id, ticks[id], nested)
}

fn document(spec: Scenario, stimulus: Stimulus) -> AnyView {
    let ticks: Vec<LocalSignal<u64>> = (0..spec.nodes).map(|_| RwSignal::new_local(0)).collect();
    let present: Vec<LocalSignal<bool>> =
        (0..spec.nodes).map(|_| RwSignal::new_local(true)).collect();
    let root = match spec.topology {
        Topology::Flat => AnyView::new(
            zgui::elements::row().class("root").children(
                (0..spec.nodes)
                    .map(|id| {
                        if matches!(spec.workload, Workload::StructuralUpdate) {
                            let present = present[id];
                            let spec = spec.clone();
                            let tick = ticks[id];
                            AnyView::new(move || {
                                present.get().then(|| item(&spec, id, tick, Vec::new()))
                            })
                        } else {
                            item(&spec, id, ticks[id], Vec::new())
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
        ),
        Topology::Wide => {
            let children = (1..spec.nodes)
                .map(|id| item(&spec, id, ticks[id], Vec::new()))
                .collect();
            item(&spec, 0, ticks[0], children)
        }
        Topology::Balanced => {
            let mut children = vec![Vec::new(); spec.nodes];
            for id in 1..spec.nodes {
                children[(id - 1) / 8].push(id);
            }
            balanced(&spec, 0, &children, &ticks)
        }
        Topology::Deep => {
            let mut current = item(&spec, spec.nodes - 1, ticks[spec.nodes - 1], Vec::new());
            for id in (0..spec.nodes - 1).rev() {
                current = item(&spec, id, ticks[id], vec![current]);
            }
            current
        }
    };
    let update_spec = spec.clone();
    *stimulus.borrow_mut() = Some(Box::new(move |sequence| {
        if matches!(update_spec.workload, Workload::StructuralUpdate) {
            let remove = sequence % 2 == 1;
            let count = (update_spec.nodes / 10).max(1);
            for signal in &present[update_spec.nodes - count..] {
                signal.set(!remove);
            }
        } else if !matches!(
            update_spec.workload,
            Workload::Idle | Workload::ColdStart | Workload::ResizeStep | Workload::ResizeStorm
        ) {
            for index in bench_core::affected_indices(&update_spec, sequence) {
                ticks[index].set(sequence);
            }
        }
    }));
    root
}

struct Probe {
    spec: Scenario,
    mode: MeasurementMode,
    epoch: Instant,
    sequence: Rc<Cell<u64>>,
    last_frame: Cell<Option<Instant>>,
    stimulus_at: Cell<Option<Instant>>,
    stimulus: Stimulus,
    surface: Arc<Mutex<Option<Arc<dyn Surface>>>>,
    done: Rc<Cell<bool>>,
}

fn mark_delta(marks: &[zgui_profile::latency::Recorded], from: &str, to: &str) -> Option<u64> {
    let a = marks.iter().find(|m| m.stage == from)?.at_ns;
    let b = marks.iter().find(|m| m.stage == to)?.at_ns;
    b.checked_sub(a).and_then(|v| u64::try_from(v).ok())
}

impl FrameProbe for Probe {
    fn frame_ended(&self, window: &Window) {
        let now = Instant::now();
        let sequence = self.sequence.get();
        let all = if self.mode == MeasurementMode::Diagnostic {
            zgui_profile::latency::last(160)
        } else {
            Vec::new()
        };
        let marks = all
            .iter()
            .rposition(|m| m.stage == "f.begin")
            .map(|i| &all[i..])
            .unwrap_or(&[]);
        let memory = window.renderer().memory();
        let mut details = BTreeMap::new();
        details.insert("fixed".into(), memory.fixed);
        details.insert("targets".into(), memory.targets);
        details.insert("scratch".into(), memory.scratch);
        details.insert("atlases".into(), memory.atlases);
        details.insert("buffers".into(), memory.buffers);
        let size = window.renderer().target().map(|v| v.size);
        let phase = if sequence == 0 {
            SamplePhase::Startup
        } else if sequence <= self.spec.warmup_frames as u64 {
            SamplePhase::Warmup
        } else {
            SamplePhase::Measurement
        };
        let sample = FrameSample {
            sequence,
            phase,
            timestamp_ns: self.epoch.elapsed().as_nanos() as u64,
            frame_time_ns: self
                .last_frame
                .replace(Some(now))
                .map(|v| now.duration_since(v).as_nanos() as u64),
            stimulus_latency_ns: self
                .stimulus_at
                .take()
                .map(|v| now.duration_since(v).as_nanos() as u64),
            observed_width: size.map(|s| s.width.max(0) as u32),
            observed_height: size.map(|s| s.height.max(0) as u32),
            pipeline: if self.mode == MeasurementMode::Diagnostic {
                PipelineSample {
                    view_build_ns: mark_delta(marks, "f.flush", "f.commands"),
                    style_ns: mark_delta(marks, "f.restyle", "f.brushes"),
                    layout_ns: mark_delta(marks, "f.layout", "f.enter"),
                    prepaint_ns: mark_delta(marks, "f.boxes", "f.layout"),
                    paint_ns: mark_delta(marks, "p.emit", "p.finish"),
                    gpu_prepare_ns: mark_delta(marks, "draw.in", "r.record"),
                    command_encode_ns: mark_delta(marks, "r.record", "acq.in"),
                    queue_submit_ns: mark_delta(marks, "acq.out", "sub.out"),
                    present_ns: mark_delta(marks, "sub.out", "pres.out"),
                    device_gpu_timestamps: false,
                }
            } else {
                PipelineSample::default()
            },
            framework_memory: Some(FrameworkMemory {
                cpu_bytes: None,
                gpu_bytes: Some(memory.total()),
                atlas_bytes: Some(memory.atlases),
                details,
            }),
            valid: true,
            note: None,
        };
        let _ = bench_core::emit_event(&AdapterEvent::Frame { sample });
        let total = self.spec.warmup_frames as u64 + self.spec.sample_frames as u64;
        if sequence >= total {
            self.done.set(true);
            return;
        }
        let next = sequence + 1;
        self.sequence.set(next);
        self.stimulus_at.set(Some(Instant::now()));
        if let Some(update) = self.stimulus.borrow().as_ref() {
            update(next)
        }
        if let Ok(surface) = self.surface.lock() {
            if let Some(surface) = surface.as_ref() {
                match self.spec.workload {
                    Workload::ResizeStep => {
                        let large = (sequence / 8) % 2 == 0;
                        let _ = surface.request_size(zgui::geom::Size::new(
                            zgui::geom::CssPx(if large { 1200.0 } else { 900.0 }),
                            zgui::geom::CssPx(if large { 800.0 } else { 620.0 }),
                        ));
                    }
                    Workload::ResizeStorm => {
                        let d = (sequence % 120) as f32;
                        let _ = surface.request_size(zgui::geom::Size::new(
                            zgui::geom::CssPx(840.0 + d * 3.0),
                            zgui::geom::CssPx(560.0 + d * 2.0),
                        ));
                    }
                    _ => surface.request_redraw(),
                }
            }
        }
    }
    fn describe(&self) -> &str {
        "gui-bench recorder"
    }
}

struct ExitHandler {
    inner: Box<dyn AppHandler>,
    done: Rc<Cell<bool>>,
    frames: Rc<Cell<u64>>,
}
impl AppHandler for ExitHandler {
    fn surfaces_available(&mut self, cx: &dyn PlatformCx) {
        self.inner.surfaces_available(cx)
    }
    fn surfaces_lost(&mut self, cx: &dyn PlatformCx) {
        self.inner.surfaces_lost(cx)
    }
    fn surface_event(&mut self, cx: &dyn PlatformCx, surface: SurfaceId, event: SurfaceEvent) {
        self.inner.surface_event(cx, surface, event)
    }
    fn wake(&mut self, cx: &dyn PlatformCx, reason: WakeReason) {
        self.inner.wake(cx, reason)
    }
    fn idle(&mut self, cx: &dyn PlatformCx) -> IdlePolicy {
        let policy = self.inner.idle(cx);
        if self.done.get() {
            let _ = bench_core::emit_event(&AdapterEvent::Complete {
                frames: self.frames.get() + 1,
            });
            cx.request_exit()
        }
        policy
    }
    fn deadline_reached(&mut self, cx: &dyn PlatformCx) {
        self.inner.deadline_reached(cx)
    }
    fn shutting_down(&mut self, cx: &dyn PlatformCx) {
        self.inner.shutting_down(cx)
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let spec: Scenario = serde_json::from_str(&args.scenario).context("invalid scenario JSON")?;
    spec.validate().map_err(anyhow::Error::msg)?;
    let mode: MeasurementMode = args.mode.into();
    bench_core::emit_event(&AdapterEvent::Hello {
        protocol_version: bench_core::PROTOCOL_VERSION,
        framework: Framework::Zgui,
        framework_version: "zgui 0.1.0 (local path)".into(),
        mode,
        pid: std::process::id(),
        capabilities: vec![
            "frame_probe".into(),
            "stimulus_to_frame_latency".into(),
            "latency_marks".into(),
            "internal_gpu_memory".into(),
            "window_resize".into(),
            "pipeline_cpu_stages".into(),
        ],
    })?;
    let mut start = String::new();
    std::io::stdin().lock().read_line(&mut start)?;
    if start.trim() != "start" {
        anyhow::bail!("expected start command")
    }
    bench_core::emit_event(&AdapterEvent::Ready)?;
    let epoch = Instant::now();
    if mode == MeasurementMode::Diagnostic {
        zgui_profile::latency::retain(16_384);
        zgui_profile::latency::clear();
    }
    let fonts = Fonts::shipped_only();
    fonts.register(Arc::new(FONT.to_vec()), Some("IBM Plex Sans"))?;
    let stimulus: Stimulus = Rc::new(RefCell::new(None));
    let surface: Arc<Mutex<Option<Arc<dyn Surface>>>> = Arc::new(Mutex::new(None));
    let done = Rc::new(Cell::new(false));
    let sequence = Rc::new(Cell::new(0));
    let probe = Rc::new(Probe {
        spec: spec.clone(),
        mode,
        epoch,
        sequence: sequence.clone(),
        last_frame: Cell::new(None),
        stimulus_at: Cell::new(None),
        stimulus: stimulus.clone(),
        surface: surface.clone(),
        done: done.clone(),
    });
    let surface_for_renderer = surface.clone();
    let renderer = Box::new(
        move |native: &Arc<dyn Surface>, target: zgui_render::RenderTarget| {
            *surface_for_renderer.lock().unwrap() = Some(native.clone());
            let handles = Arc::clone(native).gpu_shared().ok_or_else(|| {
                zgui_runtime::AppError::Platform(zgui_platform::PlatformError::Backend(
                    "surface exposes no GPU handles".into(),
                ))
            })?;
            let builder = zgui_render_wgpu::Builder::new();
            let drawable = builder
                .instance()
                .create_surface(handles)
                .map_err(|e| zgui_platform::PlatformError::Backend(e.to_string()))?;
            Ok(Box::new(builder.for_surface(target, drawable)?) as Box<dyn zgui_render::Renderer>)
        },
    );
    let app = zgui::app()
        .with_title("zgui benchmark")
        .with_size(spec.width as f32, spec.height as f32)
        .with_stylesheet(SHEET)
        .with_fonts(fonts)
        .with_renderer(renderer)
        .with_probe(probe);
    let spec_for_view = spec.clone();
    app.run_on(
        |handler| {
            zgui::app::desktop()(Box::new(ExitHandler {
                inner: handler,
                done: done.clone(),
                frames: sequence.clone(),
            }))
        },
        move || document(spec_for_view.clone(), stimulus.clone()),
    )?;
    zgui_profile::latency::retain(0);
    Ok(())
}
