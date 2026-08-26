use anyhow::{Context as _, Result};
use bench_core::{
    AdapterEvent, FrameSample, Framework, MeasurementMode, PipelineSample, SamplePhase, Scenario,
    StyleComplexity, Topology, Workload,
};
use clap::{Parser, ValueEnum};
use gpui::{
    AnyElement, App, Bounds, Context, Entity, IntoElement, ParentElement, Render, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::io::BufRead;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

const FONT: &[u8] = include_bytes!("../../../assets/IBMPlexSans-Regular.ttf");

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

struct BenchView {
    spec: Scenario,
    tick: u64,
    active: Vec<bool>,
    last_active: Vec<usize>,
    view_build_ns: Arc<AtomicU64>,
}

impl BenchView {
    fn stimulate(&mut self) {
        for &index in &self.last_active {
            self.active[index] = false;
        }
        self.tick += 1;
        self.last_active = bench_core::affected_indices(&self.spec, self.tick);
        for &index in &self.last_active {
            self.active[index] = true;
        }
    }

    fn item(&self, id: usize, children: Vec<AnyElement>) -> AnyElement {
        let changed = self.active[id];
        let tick = self.tick;
        let mut item = div()
            .flex()
            .flex_col()
            .min_w(px(18.0))
            .min_h(px(16.0))
            .px_1()
            .bg(
                if changed
                    && matches!(
                        self.spec.workload,
                        Workload::PaintUpdate | Workload::InputLatency
                    )
                {
                    rgb(0x357d74)
                } else {
                    rgb(0x18243a)
                },
            )
            .text_color(rgb(0xe8eefc))
            .font_family("IBM Plex Sans")
            .text_sm();
        if matches!(self.spec.workload, Workload::LayoutUpdate) && changed {
            item = item.pl_3().min_w(px(30.0 + (tick % 7) as f32));
        }
        if self.spec.style == StyleComplexity::Decorated {
            item = item
                .border_1()
                .border_color(rgb(0x3b4d6b))
                .rounded_sm()
                .shadow_sm();
        }
        let label = if matches!(self.spec.workload, Workload::TextUpdate) && changed {
            format!("item {id:05} · {tick:04}")
        } else {
            format!("item {id:05}")
        };
        item.child(label).children(children).into_any_element()
    }

    fn balanced(&self, id: usize, children: &[Vec<usize>], visible: usize) -> AnyElement {
        let nested = children[id]
            .iter()
            .copied()
            .filter(|&c| c < visible)
            .map(|c| self.balanced(c, children, visible))
            .collect();
        self.item(id, nested)
    }

    fn tree(&self) -> AnyElement {
        let visible =
            if matches!(self.spec.workload, Workload::StructuralUpdate) && self.tick % 2 == 1 {
                self.spec
                    .nodes
                    .saturating_sub((self.spec.nodes / 10).max(1))
            } else {
                self.spec.nodes
            };
        match self.spec.topology {
            Topology::Flat => div()
                .flex()
                .flex_wrap()
                .gap_1()
                .children((0..visible).map(|id| self.item(id, Vec::new())))
                .into_any_element(),
            Topology::Wide => {
                if visible == 0 {
                    return div().into_any_element();
                }
                self.item(
                    0,
                    (1..visible).map(|id| self.item(id, Vec::new())).collect(),
                )
            }
            Topology::Balanced => {
                let mut children = vec![Vec::new(); visible];
                for id in 1..visible {
                    children[(id - 1) / 8].push(id);
                }
                self.balanced(0, &children, visible)
            }
            Topology::Deep => {
                let mut current = self.item(visible.saturating_sub(1), Vec::new());
                for id in (0..visible.saturating_sub(1)).rev() {
                    current = self.item(id, vec![current]);
                }
                current
            }
        }
    }
}

impl Render for BenchView {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // GPUI's animation-frame request must be made while a view is on the
        // rendered-entity stack. Passive workloads use it to keep sampling
        // frames without manufacturing application updates.
        if matches!(self.spec.workload, Workload::Idle | Workload::ColdStart) {
            window.request_animation_frame();
        }
        let started = Instant::now();
        let tree = self.tree();
        self.view_build_ns
            .store(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
        div()
            .size_full()
            .overflow_hidden()
            .bg(rgb(0x0b1020))
            .p_2()
            .child(tree)
    }
}

struct Driver {
    spec: Scenario,
    mode: MeasurementMode,
    epoch: Instant,
    sequence: Cell<u64>,
    last_frame: Cell<Option<Instant>>,
    stimulus: Cell<Option<Instant>>,
    frame_collector: RefCell<gpui::profiler::FrameTimingCollector>,
    renderer_collector: RefCell<gpui::profiler::RendererTimingCollector>,
    view_build_ns: Arc<AtomicU64>,
}

fn drive(window: &Window, view: Entity<BenchView>, driver: Rc<Driver>) {
    window.on_next_frame(move |window, cx| {
        let now = Instant::now();
        let sequence = driver.sequence.get();
        let timings = driver.frame_collector.borrow_mut().collect_unseen();
        let renderers = driver.renderer_collector.borrow_mut().collect_unseen();
        let timing = timings.last();
        let renderer = renderers.last();
        let phase = if sequence == 0 {
            SamplePhase::Startup
        } else if sequence <= driver.spec.warmup_frames as u64 {
            SamplePhase::Warmup
        } else {
            SamplePhase::Measurement
        };
        let viewport = window.viewport_size();
        let diagnostic = driver.mode == MeasurementMode::Diagnostic;
        let sample = FrameSample {
            sequence,
            phase,
            timestamp_ns: driver.epoch.elapsed().as_nanos() as u64,
            frame_time_ns: driver
                .last_frame
                .replace(Some(now))
                .map(|v| now.duration_since(v).as_nanos() as u64),
            stimulus_latency_ns: driver
                .stimulus
                .take()
                .map(|v| now.duration_since(v).as_nanos() as u64),
            observed_width: Some(u32::from(viewport.width)),
            observed_height: Some(u32::from(viewport.height)),
            pipeline: if diagnostic {
                PipelineSample {
                    view_build_ns: Some(driver.view_build_ns.load(Ordering::Relaxed)),
                    style_ns: None,
                    layout_ns: timing.map(|v| v.pipeline.compute_layout.as_nanos() as u64),
                    prepaint_ns: timing.map(|v| {
                        v.pipeline
                            .prepaint
                            .saturating_sub(v.pipeline.compute_layout)
                            .as_nanos() as u64
                    }),
                    paint_ns: timing.map(|v| v.pipeline.paint.as_nanos() as u64),
                    gpu_prepare_ns: renderer.map(|v| (v.acquire + v.prepare).as_nanos() as u64),
                    command_encode_ns: renderer.map(|v| v.encode.as_nanos() as u64),
                    queue_submit_ns: renderer.map(|v| v.submit.as_nanos() as u64),
                    present_ns: renderer.map(|v| v.present.as_nanos() as u64),
                    device_gpu_timestamps: false,
                }
            } else {
                PipelineSample::default()
            },
            framework_memory: None,
            valid: timing.is_some() || sequence == 0,
            note: (timing.is_none() && sequence > 0)
                .then(|| "no GPUI draw recorded (coalesced or occluded)".into()),
        };
        let _ = bench_core::emit_event(&AdapterEvent::Frame { sample });
        let total = driver.spec.warmup_frames as u64 + driver.spec.sample_frames as u64;
        if sequence >= total {
            let _ = bench_core::emit_event(&AdapterEvent::Complete {
                frames: sequence + 1,
            });
            cx.quit();
            return;
        }
        driver.sequence.set(sequence + 1);
        let passive = matches!(driver.spec.workload, Workload::Idle | Workload::ColdStart);
        if !passive {
            driver.stimulus.set(Some(Instant::now()));
            view.update(cx, |state, cx| {
                state.stimulate();
                cx.notify();
            });
        }
        match driver.spec.workload {
            Workload::ResizeStep => {
                let large = (sequence / 8) % 2 == 0;
                window.resize(size(
                    px(if large { 1_200.0 } else { 900.0 }),
                    px(if large { 800.0 } else { 620.0 }),
                ));
            }
            Workload::ResizeStorm => {
                let d = (sequence % 120) as f32;
                window.resize(size(px(840.0 + d * 3.0), px(560.0 + d * 2.0)));
            }
            _ => {}
        }
        drive(window, view, driver);
    });
}

fn main() -> Result<()> {
    let args = Args::parse();
    let spec: Scenario = serde_json::from_str(&args.scenario).context("invalid scenario JSON")?;
    spec.validate().map_err(anyhow::Error::msg)?;
    let mode: MeasurementMode = args.mode.into();
    bench_core::emit_event(&AdapterEvent::Hello {
        protocol_version: bench_core::PROTOCOL_VERSION,
        framework: Framework::Gpui,
        framework_version: "GPUI-CE 0.2.2 (vendored)".into(),
        mode,
        pid: std::process::id(),
        capabilities: vec![
            "frame_timing".into(),
            "stimulus_to_frame_latency".into(),
            "window_resize".into(),
            "pipeline_cpu_stages".into(),
            "wgpu_submission_stages".into(),
        ],
    })?;
    let mut start = String::new();
    std::io::stdin().lock().read_line(&mut start)?;
    if start.trim() != "start" {
        anyhow::bail!("expected start command")
    }
    bench_core::emit_event(&AdapterEvent::Ready)?;
    let epoch = Instant::now();
    gpui::profiler::set_frame_trace_enabled(true);
    gpui::profiler::set_pipeline_detail_enabled(mode == MeasurementMode::Diagnostic);
    let view_build_ns = Arc::new(AtomicU64::new(0));
    gpui_platform::application().run(move |cx: &mut App| {
        cx.text_system()
            .add_fonts(vec![Cow::Borrowed(FONT)])
            .expect("register benchmark font");
        let bounds = Bounds::centered(
            None,
            size(px(spec.width as f32), px(spec.height as f32)),
            cx,
        );
        let driver = Rc::new(Driver {
            spec: spec.clone(),
            mode,
            epoch,
            sequence: Cell::new(0),
            last_frame: Cell::new(None),
            stimulus: Cell::new(None),
            frame_collector: RefCell::new(gpui::profiler::FrameTimingCollector::new()),
            renderer_collector: RefCell::new(gpui::profiler::RendererTimingCollector::new()),
            view_build_ns: view_build_ns.clone(),
        });
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let entity = cx.new(|_| BenchView {
                    spec: spec.clone(),
                    tick: 0,
                    active: vec![false; spec.nodes],
                    last_active: Vec::new(),
                    view_build_ns: view_build_ns.clone(),
                });
                drive(window, entity.clone(), driver.clone());
                entity
            },
        )
        .expect("open GPUI benchmark window");
        cx.activate(true);
    });
    Ok(())
}
