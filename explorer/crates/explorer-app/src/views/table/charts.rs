//! The charts of the table view.

use explorer_core::api::{CellInfo, MetricRef};
use explorer_core::scale::ScaleKind;
use explorer_core::stats::ratio_ci;
use explorer_core::units::short;
use leptos::prelude::*;

use super::view::{OPS, op_metric};
use crate::chart::{
    Axis, ChartCard, DotModel, DotPlot, DotRow, HeatCell, HeatModel, Heatmap, Rule, spread_note,
    use_card,
};
use crate::palette;
use crate::state::{Selection, use_explorer};
use crate::views::kit::{Build, Data, baseline_label};
use crate::widgets::Segmented;

/// One table operation as a dot plot of the shown frameworks and variants.
#[component]
pub fn OpDots(
    data: Data,
    op: &'static str,
    title: &'static str,
    rows: Memo<String>,
    measure: Memo<String>,
) -> impl IntoView {
    view! {
        <ChartCard title=title>
            <OpPlot data=data op=op rows=rows measure=measure/>
        </ChartCard>
    }
}

#[component]
fn OpPlot(
    data: Data,
    op: &'static str,
    rows: Memo<String>,
    measure: Memo<String>,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let d = data.clone();
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        let m = op_metric(op, &measure.get());
        ctx.note
            .set(spread_note(d.samples(&m), ctx.uncertainty(), "Dot"));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let m = op_metric(op, &measure.get());
        let n: u32 = rows.get().parse().unwrap_or(1000);
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let rows = b.dot_rows("table", &m, |k| k.rows == n);
        let base = prefs.base.is_some();
        let words = if measure.get() == "cpu" {
            "CPU ms per operation"
        } else {
            "milliseconds"
        };
        let mut x = Axis::value(
            if base {
                baseline_label(&prefs, "")
            } else {
                words.to_owned()
            },
            if measure.get() == "cpu" {
                "ms/op"
            } else {
                "ms"
            },
        );
        if base {
            x.kind = ScaleKind::Log;
        }
        let rules = if base {
            vec![Rule {
                horizontal: false,
                value: 1.0,
                label: String::new(),
            }]
        } else {
            Vec::new()
        };
        DotModel {
            rows,
            x,
            rules,
            hide_labels: false,
            factors: false,
        }
    });
    view! { <DotPlot model=model/> }
}

/// The speedup of one variant over another per framework and operation.
#[component]
pub fn Speedup(data: Data, rows: Memo<String>) -> impl IntoView {
    let pair = RwSignal::new("coarse".to_owned());
    let controls = ViewFn::from(move || {
        view! {
            <Segmented
                options=vec![("coarse".to_owned(), "coarse ÷ fine".to_owned()), ("shared".to_owned(), "shared ÷ fine".to_owned())]
                value=Signal::derive(move || pair.get())
                on_change=Callback::new(move |v| pair.set(v))
            />
        }
    });
    view! {
        <ChartCard
            title="Variant speedup"
            note="Median latency of the variant over the fine variant, per framework and operation, with the 95% bootstrap CI over repeats · blue: fine is faster · orange: the variant is faster · gray: the CI holds 1"
            controls=controls
            wide=true
        >
            <SpeedupMap data=data rows=rows pair=pair/>
        </ChartCard>
    }
}

#[component]
fn SpeedupMap(data: Data, rows: Memo<String>, pair: RwSignal<String>) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let n: u32 = rows.get().parse().unwrap_or(1000);
        let other = pair.get();
        let sel = Selection::new(data.overview(), &prefs, "table");
        let cells: Vec<&CellInfo> = sel
            .cells
            .iter()
            .copied()
            .filter(|c| c.key.rows == n)
            .collect();
        let fws: Vec<String> = sel
            .frameworks()
            .into_iter()
            .filter(|f| {
                cells
                    .iter()
                    .any(|c| c.key.framework == *f && c.key.variant == other)
            })
            .collect();
        let find = |fw: &str, v: &str| {
            cells
                .iter()
                .find(|c| c.key.framework == fw && c.key.variant == v)
                .copied()
        };
        let grid = fws
            .iter()
            .map(|fw| {
                OPS.iter()
                    .map(|(op, _)| {
                        let m = op_metric(op, "latency");
                        let (fine, var) = (find(fw, "fine")?, find(fw, &other)?);
                        let (a, b) = (data.stat(&m, var.id)?, data.stat(&m, fine.id)?);
                        let r = ratio_ci(&a.values, &b.values, 0.95)?;
                        let t = if r.spans_one() {
                            0.0
                        } else {
                            -(r.estimate.log2() / 3.0).clamp(-1.0, 1.0)
                        };
                        Some(HeatCell {
                            text: format!("{}×", short(r.estimate)),
                            t: Some(t),
                            cell: Some(var.id),
                            invalid: !var.is_valid(op) || !fine.is_valid(op),
                            tip: vec![
                                (format!("{other} ÷ fine"), format!("{}×", short(r.estimate))),
                                (
                                    "95% CI".to_owned(),
                                    r.lo.zip(r.hi).map_or("–".to_owned(), |(l, h)| {
                                        format!("{} – {}×", short(l), short(h))
                                    }),
                                ),
                                ("fine".to_owned(), format!("{} ms", short(b.median))),
                                (other.clone(), format!("{} ms", short(a.median))),
                            ],
                        })
                    })
                    .collect()
            })
            .collect();
        HeatModel {
            rows: fws.iter().map(|f| (f.clone(), palette::ink(f))).collect(),
            cols: OPS
                .iter()
                .map(|(op, _)| op.trim_start_matches("op-").to_owned())
                .collect(),
            cells: grid,
            diverging: true,
            ends: ("variant faster".to_owned(), "fine faster".to_owned()),
        }
    });
    view! { <Heatmap model=model/> }
}

/// The memory that a churn of replacements keeps per cycle.
#[component]
pub fn Churn(data: Data) -> impl IntoView {
    let what = RwSignal::new("pss_kept_per_cycle".to_owned());
    let controls = ViewFn::from(move || {
        view! {
            <Segmented
                options=vec![
                    ("pss_kept_per_cycle".to_owned(), "PSS".to_owned()),
                    ("private_kept_per_cycle".to_owned(), "private".to_owned()),
                    ("vram_kept_per_cycle".to_owned(), "VRAM".to_owned()),
                ]
                value=Signal::derive(move || what.get())
                on_change=Callback::new(move |v| what.set(v))
            />
        }
    });
    view! {
        <ChartCard title="Memory kept per replace-all cycle" note="Memory after minus before a churn of replacements of every row, per cycle · near zero: nothing leaks · one sample per cell" controls=controls wide=true>
            <ChurnPlot data=data what=what/>
        </ChartCard>
    }
}

#[component]
fn ChurnPlot(data: Data, what: RwSignal<String>) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let m = MetricRef::new("churn", "memory", &what.get());
        let b = Build::new(&data, &prefs, crate::state::Uncertainty::None);
        let mut rows = Vec::new();
        let sizes = Selection::new(data.overview(), &prefs, "table").sizes();
        for (n, _) in sizes {
            rows.push(DotRow::header(format!(
                "{} rows",
                explorer_core::units::thousands(f64::from(n))
            )));
            let mut r = b.dot_rows("table", &m, |k| k.rows == n);
            for row in &mut r {
                if let Some(v) = row.value {
                    row.text = format!("{} KiB", short(v));
                }
            }
            rows.extend(r);
        }
        let mut x = Axis::value("KiB per cycle", "KiB");
        x.zero = false;
        DotModel {
            rows,
            x,
            rules: vec![Rule {
                horizontal: false,
                value: 0.0,
                label: String::new(),
            }],
            hide_labels: false,
            factors: false,
        }
    });
    view! { <DotPlot model=model/> }
}
