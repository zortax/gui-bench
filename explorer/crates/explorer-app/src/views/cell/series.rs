//! The frame, resize and phase series of a measurement: scroll steps and frame
//! intervals over time, delivered frames per second, configure to
//! present per configure, and the durations of zgui's app phases.

use explorer_core::api::{FramesDetail, PhasesDetail, ResizeDetail};
use explorer_core::stats::{quantile, sorted};
use explorer_core::units::short;
use leptos::prelude::*;

use super::latency::iteration_color;
use crate::api::{frames_detail, phases_detail, resize_detail};
use crate::chart::{
    Axis, ChartCard, DotModel, DotPlot, DotRow, Legend, LegendItem, Pt, Rule, Series, XyModel,
    XyPlot,
};
use crate::palette::Marker;
use crate::state::use_explorer;
use crate::widgets::{ChartSkeleton, ErrorCard, Segmented};

fn series(key: String, label: String, color: String) -> Series {
    let mut s = Series::of_framework(key, "", label);
    s.color = color.clone();
    s.soft = color.clone();
    s.ink = color;
    s
}

fn iteration_picker(count: usize, picked: RwSignal<String>) -> impl IntoView {
    let options: Vec<(String, String)> = (0..count)
        .map(|i| (i.to_string(), format!("iteration {}", i + 1)))
        .collect();
    view! { <Segmented options=options value=Signal::derive(move || picked.get()) on_change=Callback::new(move |v| picked.set(v))/> }
}

/// The frame charts of a measurement.
#[component]
pub fn FrameCharts(cell: u32, measurement: String) -> impl IntoView {
    let run = use_explorer().run.get_untracked();
    let detail = Resource::new(
        move || (run.clone(), cell, measurement.clone()),
        |(run, cell, m)| frames_detail(run, cell, m),
    );
    view! {
        <Suspense fallback=|| view! { <ChartSkeleton/> }>
            {move || Suspend::new(async move {
                match detail.await {
                    Ok(d) => view! { <Frames d=d/> }.into_any(),
                    Err(e) => view! { <ErrorCard title="The frames do not load" error=e.to_string()/> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn Frames(d: FramesDetail) -> impl IntoView {
    let picked = RwSignal::new("0".to_owned());
    let refresh = d.refresh_ms;
    let scroll = d.scroll.clone();
    let n = scroll.len();
    let steps = Signal::derive(move || {
        let i: usize = picked.get().parse().unwrap_or(0);
        let mut m = XyModel::new(
            Axis::linear("time from the first frame", "ms"),
            Axis::linear("content moved per frame", "px"),
        );
        m.height = 220.0;
        m.crosshair = true;
        let Some(it) = scroll.get(i) else {
            return m;
        };
        let mut expected = series("expected".into(), "expected".into(), "var(--muted)".into());
        expected.step = true;
        expected.radius = 0.0;
        expected.dash = "3 3".into();
        let mut measured = series("measured".into(), "measured".into(), "var(--accent)".into());
        measured.step = true;
        measured.radius = 0.0;
        let mut stalls = series("stalls".into(), "stalled".into(), "var(--err)".into());
        stalls.line = false;
        stalls.radius = 3.6;
        let mut doubles = series("double".into(), "double step".into(), "var(--warn)".into());
        doubles.line = false;
        doubles.radius = 3.6;
        doubles.marker = Marker::Diamond;
        for s in &it.steps {
            expected.points.push(Pt::new(s.t_ms, s.expected_px.abs()));
            let mut p = Pt::new(s.t_ms, s.measured_px.abs());
            p.tip = vec![
                ("interval".into(), format!("{} ms", short(s.interval_ms))),
                ("missed refreshes".into(), s.vblanks_missed.to_string()),
            ];
            measured.points.push(p.clone());
            if s.stalled {
                stalls.points.push(p.clone());
            }
            if s.double_step {
                doubles.points.push(p);
            }
        }
        m.series = vec![expected, measured, stalls, doubles];
        m
    });
    let scroll = d.scroll.clone();
    let intervals = Signal::derive(move || {
        let i: usize = picked.get().parse().unwrap_or(0);
        let mut m = XyModel::new(
            Axis::linear("time from the first frame", "ms"),
            Axis::value("frame interval", "ms"),
        );
        m.height = 170.0;
        if let Some(it) = scroll.get(i) {
            let mut s = series("interval".into(), "interval".into(), "var(--seq-3)".into());
            s.radius = 2.0;
            s.points = it
                .steps
                .iter()
                .map(|x| Pt::new(x.t_ms, x.interval_ms))
                .collect();
            m.series.push(s);
        }
        m.rules.push(Rule {
            horizontal: true,
            value: refresh,
            label: format!("one refresh · {} ms", short(refresh)),
        });
        m
    });
    let mut fps = XyModel::new(
        Axis::linear("second of the measurement", "s"),
        Axis::value("frames delivered", ""),
    );
    fps.height = 170.0;
    for (i, it) in d.pacing.iter().enumerate() {
        let mut s = series(
            format!("p{i}"),
            format!("iteration {}", it.index),
            iteration_color(i),
        );
        s.points = it
            .fps_series
            .iter()
            .enumerate()
            .map(|(sec, f)| Pt::new(sec as f64, f64::from(*f)))
            .collect();
        fps.series.push(s);
    }
    fps.rules.push(Rule {
        horizontal: true,
        value: 1000.0 / refresh,
        label: "one frame per refresh".into(),
    });
    let pacing = d.pacing.clone();
    let legend = vec![
        LegendItem::encoding("expected", Marker::Circle, Some("3 3".into()), true),
        {
            let mut i =
                LegendItem::encoding("measured", Marker::Circle, Some(String::new()), false);
            i.color = "var(--accent)".into();
            i
        },
        {
            let mut i = LegendItem::encoding("stalled", Marker::Circle, None, false);
            i.color = "var(--err)".into();
            i
        },
        {
            let mut i = LegendItem::encoding("double step", Marker::Diamond, None, false);
            i.color = "var(--warn)".into();
            i
        },
    ];
    view! {
        {(n > 0).then(|| view! {
            <ChartCard
                title="Scroll steps"
                note="Content movement per presented frame against the input's expectation · ctrl+wheel or drag to zoom into frames"
                controls=ViewFn::from(move || iteration_picker(n, picked))
            >
                <Legend items=legend.clone()/>
                <XyPlot model=steps/>
            </ChartCard>
            <ChartCard title="Frame intervals" note="Time between presented frames of the glide">
                <XyPlot model=intervals/>
            </ChartCard>
        })}
        {(!d.pacing.is_empty()).then(|| view! {
            <ChartCard title="Frame pacing" note="Frames delivered in each second, per iteration">
                <XyPlot model=fps/>
                <table class="gx-table">
                    <thead><tr><th>"iteration"</th><th>"fps"</th><th>"interval p50"</th><th>"p99"</th><th>"max"</th><th>"janky"</th><th>"missed"</th><th>"discarded"</th></tr></thead>
                    <tbody>
                        {pacing.iter().map(|p| view! {
                            <tr>
                                <td>{p.index}</td>
                                <td>{short(p.fps)}</td>
                                <td>{format!("{} ms", short(p.interval.p50))}</td>
                                <td>{format!("{} ms", short(p.interval.p99))}</td>
                                <td>{format!("{} ms", short(p.interval.max))}</td>
                                <td>{p.jank_count}</td>
                                <td>{p.missed_vblanks}</td>
                                <td>{p.discarded}</td>
                            </tr>
                        }).collect_view()}
                    </tbody>
                </table>
            </ChartCard>
        })}
    }
}

/// The resize charts of a measurement.
#[component]
pub fn ResizeCharts(cell: u32, measurement: String) -> impl IntoView {
    let run = use_explorer().run.get_untracked();
    let detail = Resource::new(
        move || (run.clone(), cell, measurement.clone()),
        |(run, cell, m)| resize_detail(run, cell, m),
    );
    view! {
        <Suspense fallback=|| view! { <ChartSkeleton/> }>
            {move || Suspend::new(async move {
                match detail.await {
                    Ok(d) => view! { <Resize d=d/> }.into_any(),
                    Err(e) => view! { <ErrorCard title="The configures do not load" error=e.to_string()/> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn Resize(d: ResizeDetail) -> impl IntoView {
    let mut m = XyModel::new(
        Axis::linear("time from the first configure", "ms"),
        Axis::value("from the configure", "ms"),
    );
    m.height = 220.0;
    let stages = [
        ("ack", "var(--seq-2)"),
        ("commit", "var(--seq-3)"),
        ("present", "var(--accent)"),
    ];
    let mut legend = Vec::new();
    for (si, (name, color)) in stages.iter().enumerate() {
        let mut s = series(format!("s{si}"), (*name).into(), (*color).into());
        s.radius = 2.4;
        if let Some(it) = d.iterations.first() {
            s.points = it
                .configures
                .iter()
                .filter_map(|c| {
                    let v = match si {
                        0 => c.ack_ms,
                        1 => c.commit_ms,
                        _ => c.present_ms,
                    }?;
                    let mut p = Pt::new(c.t_ms, v);
                    p.invalid = c.superseded;
                    p.tip = vec![
                        ("size".into(), format!("{}×{}", c.width, c.height)),
                        ("superseded".into(), c.superseded.to_string()),
                    ];
                    Some(p)
                })
                .collect();
        }
        let mut item = LegendItem::encoding(
            format!("configure to {name}"),
            Marker::Circle,
            Some(String::new()),
            false,
        );
        item.color = (*color).into();
        legend.push(item);
        m.series.push(s);
    }
    m.rules.push(Rule {
        horizontal: true,
        value: d.refresh_ms,
        label: "one refresh".into(),
    });
    view! {
        <ChartCard title="Configures" note="First iteration · hollow: a newer configure superseded it">
            <Legend items=legend/>
            <XyPlot model=m/>
        </ChartCard>
    }
}

/// The app phases of a measurement, when the app reports them (zgui).
#[component]
pub fn PhaseChart(cell: u32, measurement: String) -> impl IntoView {
    let run = use_explorer().run.get_untracked();
    let detail = Resource::new(
        move || (run.clone(), cell, measurement.clone()),
        |(run, cell, m)| phases_detail(run, cell, m),
    );
    view! {
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move { detail.await.ok().flatten().map(|d| view! { <Phases d=d/> }) })}
        </Suspense>
    }
}

#[component]
fn Phases(d: PhasesDetail) -> impl IntoView {
    let mut phases = d.phases.clone();
    phases.sort_by(|a, b| b.p50_ms.total_cmp(&a.p50_ms));
    let rows: Vec<DotRow> = phases
        .iter()
        .map(|p| {
            let s = sorted(&p.durations_ms);
            let mut r = DotRow::of_framework(p.name.clone(), "zgui", p.name.clone());
            r.value = Some(p.p50_ms);
            r.band = quantile(&s, 0.25).zip(quantile(&s, 0.75));
            r.whisker = quantile(&s, 0.05).zip(quantile(&s, 0.95));
            r.text = format!("{} ms", short(p.p50_ms));
            r.tip = vec![
                ("count".into(), p.count.to_string()),
                ("mean".into(), format!("{} ms", short(p.mean_ms))),
                ("max".into(), format!("{} ms", short(p.max_ms))),
            ];
            r
        })
        .collect();
    let mut x = Axis::value("milliseconds per span", "ms");
    x.kind = explorer_core::scale::ScaleKind::Log;
    let model = DotModel {
        rows,
        x,
        rules: Vec::new(),
        hide_labels: false,
        factors: false,
    };
    view! {
        <ChartCard title="App phases" note="zgui app marks: dot median span · band: middle half · whiskers: 5th to 95th percentile">
            <DotPlot model=model/>
        </ChartCard>
    }
}
