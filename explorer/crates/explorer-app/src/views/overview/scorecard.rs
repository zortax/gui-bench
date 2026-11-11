//! The scorecard: one row per framework, one column per key metric of a scenario, each cell
//! colored by its ratio to the best framework, and the geometric mean of those ratios.

use std::sync::Arc;

use explorer_core::api::{Better, CellInfo, MetricRef, RunOverview};
use explorer_core::stats::geomean;
use explorer_core::units::{format_value, short};
use leptos::prelude::*;

use crate::chart::{ChartCard, HeatCell, HeatModel, Heatmap};
use crate::palette;
use crate::state::{Selection, use_explorer};
use crate::views::kit::{Data, WithData};
use crate::widgets::Segmented;

/// A column of the scorecard: a metric in a scenario at a preferred size.
struct Column {
    title: &'static str,
    scenario: &'static str,
    metric: (&'static str, &'static str, &'static str),
    rows: &'static [u32],
    updates: Option<u32>,
}

const COLUMNS: &[Column] = &[
    Column {
        title: "startup\n1k rows",
        scenario: "rows",
        metric: ("startup", "startup", "first_present_ms"),
        rows: &[1000, 300, 100],
        updates: None,
    },
    Column {
        title: "hover\n1k rows",
        scenario: "rows",
        metric: ("hover", "latency", "total_ms"),
        rows: &[1000, 300, 100],
        updates: None,
    },
    Column {
        title: "hover\n10k rows",
        scenario: "rows",
        metric: ("hover", "latency", "total_ms"),
        rows: &[10000, 3000],
        updates: None,
    },
    Column {
        title: "scroll\nsmoothness 1k",
        scenario: "rows",
        metric: ("scroll", "scroll_smoothness", "smoothness_score"),
        rows: &[1000, 300, 100],
        updates: None,
    },
    Column {
        title: "table swap\n1k",
        scenario: "table",
        metric: ("op-swap", "latency", "total_ms"),
        rows: &[1000],
        updates: None,
    },
    Column {
        title: "table replace\n1k",
        scenario: "table",
        metric: ("op-replace", "latency", "total_ms"),
        rows: &[1000],
        updates: None,
    },
    Column {
        title: "ticker CPU\n1k, 1/tick",
        scenario: "ticker",
        metric: ("pacing", "resources", "cpu_ms_per_s"),
        rows: &[1000],
        updates: Some(1),
    },
    Column {
        title: "spinner GPU\nno rows",
        scenario: "spinner",
        metric: ("pacing", "frame_cost", "gpu_fence_ms_per_s"),
        rows: &[0],
        updates: None,
    },
    Column {
        title: "typing\n1k rows",
        scenario: "typing",
        metric: ("typing", "typing", "latency_p50"),
        rows: &[1000],
        updates: None,
    },
    Column {
        title: "filter\n1k rows",
        scenario: "search",
        metric: ("filter", "latency", "total_ms"),
        rows: &[1000],
        updates: None,
    },
    Column {
        title: "virtual scroll\n1M rows",
        scenario: "virtual",
        metric: ("scroll", "scroll_smoothness", "smoothness_score"),
        rows: &[1_000_000, 100_000],
        updates: None,
    },
    Column {
        title: "idle CPU\n1k rows",
        scenario: "rows",
        metric: ("idle", "resources", "cpu_ms_per_s"),
        rows: &[1000, 300, 100],
        updates: None,
    },
    Column {
        title: "memory\n1k rows",
        scenario: "rows",
        metric: ("idle", "resources", "pss_max"),
        rows: &[1000, 300, 100],
        updates: None,
    },
];

fn metric(c: &Column) -> MetricRef {
    MetricRef::new(c.metric.0, c.metric.1, c.metric.2)
}

/// Returns the cell of a framework for a column: the first preferred size, the lowest rate and
/// the finest variant.
fn pick<'a>(cells: &[&'a CellInfo], fw: &str, c: &Column) -> Option<&'a CellInfo> {
    let mine: Vec<&CellInfo> = cells
        .iter()
        .copied()
        .filter(|x| {
            x.key.framework == fw
                && x.key.scenario == c.scenario
                && (c.updates.is_none() || x.key.updates == c.updates)
        })
        .collect();
    let rows = c
        .rows
        .iter()
        .find(|r| mine.iter().any(|x| x.key.rows == **r))?;
    mine.into_iter()
        .filter(|x| x.key.rows == *rows)
        .min_by(|a, b| {
            a.key.hz.total_cmp(&b.key.hz).then(
                palette::variant_rank(&a.key.variant).cmp(&palette::variant_rank(&b.key.variant)),
            )
        })
}

/// The scorecard of a run.
#[component]
pub fn Scorecard(overview: Arc<RunOverview>) -> impl IntoView {
    let metrics: Vec<MetricRef> = COLUMNS.iter().map(metric).collect();
    let order = RwSignal::new("plot".to_owned());
    let controls = ViewFn::from(move || {
        view! {
            <Segmented
                options=vec![("plot".to_owned(), "plot order".to_owned()), ("best".to_owned(), "best first".to_owned())]
                value=Signal::derive(move || order.get())
                on_change=Callback::new(move |v| order.set(v))
            />
        }
    });
    view! {
        <ChartCard
            title="Scorecard"
            note="Each cell: the median, colored by its ratio to the best framework in the column (light: best) · score: geometric mean of the ratios · lowest refresh rate, finest variant"
            controls=controls
            wide=true
        >
            <WithData overview=overview metrics=metrics skeletons=1 view=move |data| view! { <ScoreMap data=data order=order/> }/>
        </ChartCard>
    }
}

#[component]
fn ScoreMap(data: Data, order: RwSignal<String>) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let mut cells: Vec<&CellInfo> = Vec::new();
        let mut fws: Vec<String> = Vec::new();
        for c in COLUMNS {
            let sel = Selection::new(data.overview(), &prefs, c.scenario);
            for f in sel.frameworks() {
                if !fws.contains(&f) {
                    fws.push(f);
                }
            }
            cells.extend(sel.cells);
        }
        palette::sort(&mut fws);
        let columns: Vec<&Column> = COLUMNS
            .iter()
            .filter(|c| {
                fws.iter().any(|f| {
                    pick(&cells, f, c)
                        .and_then(|x| data.stat(&metric(c), x.id))
                        .is_some()
                })
            })
            .collect();
        let mut grid: Vec<Vec<Option<HeatCell>>> = vec![Vec::new(); fws.len()];
        let mut ratios: Vec<Vec<f64>> = vec![Vec::new(); fws.len()];
        for c in &columns {
            let m = metric(c);
            let col = data.run.col(&m);
            let better = col.map_or(Better::Lower, |c| c.better);
            let unit = col.map(|c| c.unit.clone()).unwrap_or_default();
            let values: Vec<Option<(&CellInfo, f64)>> = fws
                .iter()
                .map(|f| {
                    let x = pick(&cells, f, c)?;
                    if prefs.valid_only && !x.is_valid(&m.measurement) {
                        return None;
                    }
                    Some((x, data.stat(&m, x.id)?.median))
                })
                .collect();
            let finite: Vec<f64> = values
                .iter()
                .flatten()
                .map(|(_, v)| *v)
                .filter(|v| *v > 0.0)
                .collect();
            let best = match better {
                Better::Higher => finite.iter().copied().fold(f64::MIN, f64::max),
                _ => finite.iter().copied().fold(f64::MAX, f64::min),
            };
            let ratio = |v: f64| match better {
                Better::Higher => best / v,
                _ => v / best,
            };
            let worst = finite.iter().map(|v| ratio(*v)).fold(1.0, f64::max);
            for (i, v) in values.into_iter().enumerate() {
                grid[i].push(v.map(|(x, value)| {
                    let r = if value > 0.0 { ratio(value) } else { f64::NAN };
                    if r.is_finite() {
                        ratios[i].push(r);
                    }
                    let t = if worst > 1.0 && r.is_finite() {
                        r.ln() / worst.ln()
                    } else {
                        0.0
                    };
                    HeatCell {
                        text: short(value * explorer_core::units::display(&unit).factor),
                        t: Some(t),
                        cell: Some(x.id),
                        invalid: !x.is_valid(&m.measurement),
                        tip: vec![
                            ("value".to_owned(), format_value(value, &unit)),
                            (
                                "to the best".to_owned(),
                                if r.is_finite() {
                                    format!("{}×", short(r))
                                } else {
                                    "–".to_owned()
                                },
                            ),
                            (
                                "cell".to_owned(),
                                format!("{} · {} · {} Hz", x.key.label(), x.key.size(), x.key.hz),
                            ),
                        ],
                    }
                }));
            }
        }
        let scores: Vec<Option<f64>> = ratios.iter().map(|r| geomean(r)).collect();
        let worst = scores.iter().flatten().copied().fold(1.0, f64::max);
        for (i, s) in scores.iter().enumerate() {
            grid[i].push(s.map(|s| HeatCell {
                text: format!("{}×", short(s)),
                t: Some(if worst > 1.0 {
                    s.ln() / worst.ln()
                } else {
                    0.0
                }),
                cell: None,
                invalid: false,
                tip: vec![
                    (
                        "score".to_owned(),
                        format!("{}× the best, geometric mean", short(s)),
                    ),
                    ("columns".to_owned(), ratios[i].len().to_string()),
                ],
            }));
        }
        let mut order_idx: Vec<usize> = (0..fws.len()).collect();
        if order.get() == "best" {
            order_idx.sort_by(|a, b| {
                scores[*a]
                    .unwrap_or(f64::MAX)
                    .total_cmp(&scores[*b].unwrap_or(f64::MAX))
            });
        }
        let mut cols: Vec<String> = columns.iter().map(|c| c.title.to_owned()).collect();
        cols.push("score\n× best".to_owned());
        HeatModel {
            rows: order_idx
                .iter()
                .map(|i| (fws[*i].clone(), palette::ink(&fws[*i])))
                .collect(),
            cells: order_idx.iter().map(|i| grid[*i].clone()).collect(),
            cols,
            diverging: false,
            ends: ("best".to_owned(), "furthest from the best".to_owned()),
        }
    });
    view! { <Heatmap model=model/> }
}
