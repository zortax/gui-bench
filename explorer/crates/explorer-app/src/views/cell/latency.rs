//! The latency series of a measurement: every repeat per iteration against the input phase,
//! the stages from input to presentation, the latency per input phase, and the histogram.

use explorer_core::api::LatencyDetail;
use explorer_core::units::short;
use leptos::prelude::*;

use crate::api::latency_detail;
use crate::chart::{Axis, Bar, ChartCard, Legend, LegendItem, Pt, Rule, Series, XyModel, XyPlot};
use crate::palette::Marker;
use crate::state::use_explorer;
use crate::widgets::{ChartSkeleton, ErrorCard};

/// The colors of iterations.
pub fn iteration_color(i: usize) -> String {
    const SLOTS: [&str; 6] = ["0", "5", "2", "3", "6", "4"];
    format!("var(--fw-{}-mark)", SLOTS[i % SLOTS.len()])
}

/// The latency charts of a measurement.
#[component]
pub fn LatencyCharts(cell: u32, measurement: String) -> impl IntoView {
    let explorer = use_explorer();
    let run = explorer.run.get_untracked();
    let detail = Resource::new(
        move || (run.clone(), cell, measurement.clone()),
        |(run, cell, m)| latency_detail(run, cell, m),
    );
    view! {
        <Suspense fallback=|| view! { <ChartSkeleton/> }>
            {move || Suspend::new(async move {
                match detail.await {
                    Ok(d) => view! { <Charts d=d/> }.into_any(),
                    Err(e) => view! { <ErrorCard title="The latency series does not load" error=e.to_string()/> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn Charts(d: LatencyDetail) -> impl IntoView {
    let refresh = d.refresh_ms;
    let ready = d.commit_to_ready_p50;
    let mut repeats = XyModel::new(
        Axis::linear("input phase in the refresh period", "ms"),
        Axis::value("input to photon", "ms"),
    );
    repeats.height = 220.0;
    repeats.x.domain = Some((0.0, refresh));
    let mut legend = Vec::new();
    for (i, it) in d.iterations.iter().enumerate() {
        let color = iteration_color(i);
        let mut s = Series::of_framework(
            format!("it{}", it.index),
            "",
            format!("iteration {}", it.index),
        );
        s.color = color.clone();
        s.soft = color.clone();
        s.ink = color.clone();
        s.line = false;
        s.radius = 3.4;
        s.points = it
            .samples
            .iter()
            .filter_map(|x| {
                let mut p = Pt::new(x.phase_ms, x.total_ms?);
                p.invalid = x.timed_out;
                p.tip = vec![
                    ("repeat".to_owned(), x.repeat.to_string()),
                    (
                        "input to commit".to_owned(),
                        x.input_to_commit_ms
                            .map_or("–".to_owned(), |v| format!("{} ms", short(v))),
                    ),
                    (
                        "response time".to_owned(),
                        x.input_to_commit_ms
                            .map_or("–".to_owned(), |v| format!("{} ms", short(v + ready))),
                    ),
                ];
                Some(p)
            })
            .collect();
        let mut item = LegendItem::encoding(
            format!("iteration {}", it.index),
            Marker::Circle,
            None,
            false,
        );
        item.color = color;
        legend.push(item);
        repeats.series.push(s);
    }
    repeats.rules.push(Rule {
        horizontal: true,
        value: refresh,
        label: format!("one refresh · {} ms", short(refresh)),
    });
    let mut by_phase = XyModel::new(
        Axis::linear("input phase in the refresh period", "ms"),
        Axis::value("median latency", "ms"),
    );
    by_phase.height = 180.0;
    let mut line = Series::of_framework("phase", "", "median");
    line.color = "var(--accent)".to_owned();
    line.ink = "var(--accent)".to_owned();
    line.points = d
        .by_phase
        .iter()
        .filter_map(|b| {
            let mut p = Pt::new((b.from_ms + b.to_ms) / 2.0, b.total_p50_ms?);
            p.tip = vec![("repeats".to_owned(), b.count.to_string())];
            Some(p)
        })
        .collect();
    by_phase.series.push(line);
    let mut hist = XyModel::new(
        Axis::linear("input to photon", "ms"),
        Axis::value("repeats", ""),
    );
    hist.height = 180.0;
    hist.bars = d
        .histogram
        .iter()
        .filter(|b| b.count > 0)
        .map(|b| Bar {
            x0: b.lo,
            x1: b.hi,
            y0: 0.0,
            y1: f64::from(b.count),
            color: "var(--seq-3)".to_owned(),
            group: String::new(),
            tip: vec![
                (
                    "range".to_owned(),
                    format!("{} – {} ms", short(b.lo), short(b.hi)),
                ),
                ("repeats".to_owned(), b.count.to_string()),
            ],
            cell: None,
        })
        .collect();
    let mut stages = XyModel::new(
        Axis::value("milliseconds", "ms").valued(),
        Axis::categories(vec![
            ("p50".to_owned(), "var(--text-2)".to_owned()),
            ("p95".to_owned(), "var(--text-2)".to_owned()),
        ]),
    );
    stages.x.value = true;
    stages.y.value = false;
    stages.height = 110.0;
    let colors = [
        "var(--seq-4)",
        "var(--seq-3)",
        "var(--seq-2)",
        "var(--seq-1)",
    ];
    let mut stage_legend = Vec::new();
    for (row, pick) in [(0.0, 0usize), (1.0, 1usize)] {
        let mut start = 0.0;
        for (i, s) in d.stages.iter().enumerate() {
            let v = if pick == 0 {
                s.summary.p50
            } else {
                s.summary.p95
            };
            stages.bars.push(Bar {
                x0: start,
                x1: start + v,
                y0: row - 0.3,
                y1: row + 0.3,
                color: colors[i % 4].to_owned(),
                group: String::new(),
                tip: vec![
                    (s.name.clone(), format!("{} ms", short(v))),
                    (
                        "p50 · p95 · max".to_owned(),
                        format!(
                            "{} · {} · {} ms",
                            short(s.summary.p50),
                            short(s.summary.p95),
                            short(s.summary.max)
                        ),
                    ),
                ],
                cell: None,
            });
            start += v;
        }
    }
    for (i, s) in d.stages.iter().enumerate() {
        let mut item = LegendItem::encoding(s.name.clone(), Marker::Square, None, false);
        item.color = colors[i % 4].to_owned();
        stage_legend.push(item);
    }
    let ci = d
        .median_ci
        .map(|(a, b)| {
            format!(
                " · median {} ms, 95% CI {} – {} ms",
                short(d.total.p50),
                short(a),
                short(b)
            )
        })
        .unwrap_or_default();
    view! {
        <ChartCard title="Every repeat" note=format!("Input to photon against where in the refresh period the input came{ci}")>
            <Legend items=legend/>
            <XyPlot model=repeats/>
        </ChartCard>
        <ChartCard title="Stages" note="Median and 95th percentile of each stage, stacked; the stack is no repeat's total">
            <Legend items=stage_legend/>
            <XyPlot model=stages/>
        </ChartCard>
        <div class="gx-grid gx-grid--narrow">
            <ChartCard title="Latency per input phase" note="Median of the repeats in each slice of the refresh period">
                <XyPlot model=by_phase/>
            </ChartCard>
            <ChartCard title="Histogram" note="Every repeat">
                <XyPlot model=hist/>
            </ChartCard>
        </div>
    }
}
