//! The charts of the rows view: latency against rows per refresh rate, the latency distribution
//! at one size, any metric against rows with the rates as dashes, and the startup phases.

use explorer_core::api::{CellInfo, MetricRef};
use explorer_core::scale::ScaleKind;
use explorer_core::units::short;
use leptos::prelude::*;

use crate::chart::Legend;
use crate::chart::LegendItem;
use crate::chart::{Axis, Bar, ChartCard, Rule, Series, XyModel, XyPlot, spread_note, use_card};
use crate::palette::{Marker, hz_dash};
use crate::state::{Selection, use_explorer};
use crate::views::kit::{Build, Data, SizePicker, baseline_label, ecdf};

/// The row counts of a scenario's shown cells.
fn row_ticks(data: &Data, scenario: &str) -> Vec<f64> {
    let mut rows: Vec<f64> = data
        .overview()
        .cells
        .iter()
        .filter(|c| c.key.scenario == scenario)
        .map(|c| f64::from(c.key.rows))
        .collect();
    rows.sort_by(f64::total_cmp);
    rows.dedup();
    rows
}

fn unit_of(data: &Data, m: &MetricRef, base: bool) -> String {
    if base {
        "×".to_owned()
    } else {
        explorer_core::units::display(&data.unit(m))
            .label
            .to_owned()
    }
}

/// Hover latency against rows at one refresh rate.
#[component]
pub fn LatencyVsRows(data: Data, scenario: String, hz: f64) -> impl IntoView {
    view! {
        <ChartCard title=format!("Hover latency · {hz} Hz")>
            <LatencyPlot data=data scenario=scenario hz=hz/>
        </ChartCard>
    }
}

#[component]
fn LatencyPlot(data: Data, scenario: String, hz: f64) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let m = MetricRef::new("hover", "latency", "total_ms");
    let samples = data.samples(&m);
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note
            .set(spread_note(samples, ctx.uncertainty(), "Line"));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let base = prefs.base.is_some();
        let mut y = Axis::value(baseline_label(&prefs, "ms"), unit_of(&data, &m, base));
        if base {
            y.kind = ScaleKind::Log;
        }
        let mut model = XyModel::new(Axis::log_fixed("rows", row_ticks(&data, &scenario)), y);
        model.series = b.rows_series(&scenario, &m, hz);
        model.height = 340.0;
        model.rules.push(if base {
            Rule {
                horizontal: true,
                value: 1.0,
                label: "baseline".to_owned(),
            }
        } else {
            Rule {
                horizontal: true,
                value: 1000.0 / hz,
                label: format!("one refresh · {:.1} ms", 1000.0 / hz),
            }
        });
        model
    });
    view! { <XyPlot model=model/> }
}

/// Any metric against rows, one series per framework, variant and refresh rate.
#[component]
pub fn ScrollMetric(
    data: Data,
    scenario: String,
    #[prop(into)] title: String,
    metric: MetricRef,
    #[prop(into)] words: String,
) -> impl IntoView {
    view! {
        <ChartCard title=title>
            <MetricPlot data=data scenario=scenario metric=metric words=words/>
        </ChartCard>
    }
}

#[component]
fn MetricPlot(data: Data, scenario: String, metric: MetricRef, words: String) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let samples = data.samples(&metric);
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        let rates = explorer.prefs.rates.get();
        let dashes = if rates.len() == 1 {
            ""
        } else {
            " · dashed: higher refresh rate"
        };
        ctx.note.set(format!(
            "{}{dashes}",
            spread_note(samples, ctx.uncertainty(), "Line")
        ));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let sel = Selection::new(data.overview(), &prefs, &scenario);
        let rates = sel.rates();
        let base = prefs.base.is_some();
        let mut y = Axis::value(
            if base {
                baseline_label(&prefs, "")
            } else {
                words.clone()
            },
            unit_of(&data, &metric, base),
        );
        if base {
            y.kind = ScaleKind::Log;
        }
        let mut model = XyModel::new(Axis::log_fixed("rows", row_ticks(&data, &scenario)), y);
        for hz in &rates {
            for mut s in b.rows_series(&scenario, &metric, *hz) {
                s.dash = hz_dash(*hz).to_owned();
                s.key = format!("{}@{hz}", s.key);
                if rates.len() > 1 {
                    s.label = format!("{} {hz} Hz", s.label);
                }
                s.end_label = rates.len() == 1 && !s.ghost;
                model.series.push(s);
            }
        }
        model
    });
    view! { <XyPlot model=model/> }
}

/// The distribution of hover latency at one size and rate, as empirical CDFs.
#[component]
pub fn Distribution(data: Data, scenario: String) -> impl IntoView {
    let explorer = use_explorer();
    let size = RwSignal::new(String::new());
    let rate = RwSignal::new(String::new());
    let sc = scenario.clone();
    let d = data.clone();
    let sizes = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        Selection::new(d.overview(), &prefs, &sc)
            .sizes()
            .into_iter()
            .map(|(r, _)| (r.to_string(), explorer_core::units::thousands(f64::from(r))))
            .collect::<Vec<_>>()
    });
    let sc = scenario.clone();
    let d = data.clone();
    let rates = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        Selection::new(d.overview(), &prefs, &sc)
            .rates()
            .into_iter()
            .map(|hz| (hz.to_string(), format!("{hz} Hz")))
            .collect::<Vec<_>>()
    });
    let controls = ViewFn::from(move || {
        view! {
            <SizePicker options=sizes value=size title="Rows"/>
            <SizePicker options=rates value=rate title="Refresh rate"/>
        }
    });
    view! {
        <ChartCard
            title="Hover latency distribution"
            note="Share of repeats at or below a latency, every repeat of every iteration · steeper is steadier"
            controls=controls
        >
            <DistributionPlot data=data scenario=scenario size=size rate=rate/>
        </ChartCard>
    }
}

#[component]
fn DistributionPlot(
    data: Data,
    scenario: String,
    size: RwSignal<String>,
    rate: RwSignal<String>,
) -> impl IntoView {
    let explorer = use_explorer();
    let m = MetricRef::new("hover", "latency", "total_ms");
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let rows: u32 = size.get().parse().unwrap_or(0);
        let hz: f64 = rate.get().parse().unwrap_or(0.0);
        let cells: Vec<&CellInfo> = Selection::new(data.overview(), &prefs, &scenario)
            .cells
            .into_iter()
            .filter(|c| c.key.rows == rows && c.key.hz == hz)
            .filter(|c| !prefs.valid_only || c.is_valid("hover"))
            .collect();
        let mut x = Axis::value("milliseconds", "ms").zero_based(false);
        x.zero = false;
        let mut y = Axis::linear("share of repeats", "");
        y.domain = Some((0.0, 1.0));
        let mut model = XyModel::new(x, y);
        model.height = 300.0;
        for c in cells {
            let Some(stat) = data.stat(&m, c.id) else {
                continue;
            };
            let mut s = Series::of_framework(c.id.to_string(), &c.key.framework, c.key.label());
            s.step = true;
            s.radius = 0.0;
            s.end_label = false;
            s.points = ecdf(&stat.values);
            for p in &mut s.points {
                p.cell = Some(c.id);
            }
            model.series.push(s);
        }
        model.rules.push(Rule {
            horizontal: false,
            value: 1000.0 / hz.max(1.0),
            label: "one refresh".to_owned(),
        });
        model
    });
    let legend = Signal::derive(move || {
        model.with(|m| {
            m.series
                .iter()
                .map(|s| LegendItem::framework(&s.group))
                .fold(Vec::<LegendItem>::new(), |mut v, i| {
                    if !v.iter().any(|x| x.group == i.group) {
                        v.push(i)
                    }
                    v
                })
        })
    });
    view! {
        <Legend items=legend/>
        <XyPlot model=model/>
    }
}

/// The startup milestones of every framework at one size, as stacked bars.
#[component]
pub fn StartupWaterfall(data: Data, scenario: String) -> impl IntoView {
    let explorer = use_explorer();
    let size = RwSignal::new(String::new());
    let sc = scenario.clone();
    let d = data.clone();
    let sizes = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        Selection::new(d.overview(), &prefs, &sc)
            .sizes()
            .into_iter()
            .map(|(r, _)| (r.to_string(), explorer_core::units::thousands(f64::from(r))))
            .collect::<Vec<_>>()
    });
    let controls =
        ViewFn::from(move || view! { <SizePicker options=sizes value=size title="Rows"/> });
    view! {
        <ChartCard title="Startup phases" note="Spawn to connect, first surface, toplevel, first buffer and first presented frame · lowest refresh rate" controls=controls>
            <WaterfallPlot data=data scenario=scenario size=size/>
        </ChartCard>
    }
}

const MILESTONES: [(&str, &str, &str); 5] = [
    ("connected_ms", "connected", "var(--seq-4)"),
    ("first_surface_ms", "first surface", "var(--seq-3)"),
    ("toplevel_ms", "toplevel", "var(--seq-2)"),
    ("first_buffer_ms", "first buffer", "var(--seq-1)"),
    ("first_present_ms", "first present", "var(--accent)"),
];

#[component]
fn WaterfallPlot(data: Data, scenario: String, size: RwSignal<String>) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let rows: u32 = size.get().parse().unwrap_or(0);
        let sel = Selection::new(data.overview(), &prefs, &scenario);
        let hz = sel.rates().first().copied().unwrap_or(60.0);
        let mut cells: Vec<&CellInfo> = sel
            .cells
            .into_iter()
            .filter(|c| c.key.rows == rows && c.key.hz == hz)
            .collect();
        cells.dedup_by(|a, b| a.key.framework == b.key.framework);
        let labels: Vec<(String, String)> = cells
            .iter()
            .map(|c| {
                (
                    c.key.framework.clone(),
                    crate::palette::ink(&c.key.framework),
                )
            })
            .collect();
        let mut model = XyModel::new(
            Axis::value("ms from spawn", "ms").valued(),
            Axis::categories(labels),
        );
        model.x.value = true;
        model.y.value = false;
        model.height = 38.0 + 25.0 * cells.len().max(1) as f64 + 16.0;
        for (i, c) in cells.iter().enumerate() {
            let mut stamps: Vec<(f64, &str, &str)> = MILESTONES
                .iter()
                .filter_map(|(k, name, color)| {
                    let v = data
                        .stat(&MetricRef::new("startup", "startup", k), c.id)?
                        .median;
                    Some((v, *name, *color))
                })
                .collect();
            stamps.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut start = 0.0;
            for (v, name, color) in stamps {
                model.bars.push(Bar {
                    x0: start,
                    x1: v,
                    y0: i as f64 - 0.32,
                    y1: i as f64 + 0.32,
                    color: color.to_owned(),
                    group: c.key.framework.clone(),
                    tip: vec![
                        (c.key.framework.clone(), String::new()),
                        (name.to_owned(), format!("{} ms", short(v))),
                        ("phase".to_owned(), format!("{} ms", short(v - start))),
                    ],
                    cell: Some(c.id),
                });
                start = v;
            }
        }
        model
    });
    let legend: Vec<LegendItem> = MILESTONES
        .iter()
        .map(|(_, name, color)| {
            let mut item = LegendItem::encoding(*name, Marker::Square, None, false);
            item.color = (*color).to_owned();
            item
        })
        .collect();
    view! {
        <Legend items=legend/>
        <XyPlot model=model/>
    }
}
