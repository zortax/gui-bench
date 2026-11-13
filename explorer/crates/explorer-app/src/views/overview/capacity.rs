//! The capacity of every framework: for each interaction of the app, the elements up to which
//! the 95th percentile of the response time stays within one frame, and up to which scrolling
//! and animation miss at most 1 % of the refreshes. One card per frame budget.

use std::sync::Arc;

use explorer_core::api::{CellInfo, RunOverview};
use explorer_core::stats::{quantile, sorted};
use explorer_core::story::{BUDGETS, Capacity, PACED, RESPONSES, SMOOTH_SHARE, capacity, shade};
use explorer_core::units::{compact, short};
use leptos::prelude::*;

use crate::chart::{ChartCard, HeatCell, HeatModel, Heatmap};
use crate::palette;
use crate::state::{Uncertainty, stop_of, use_explorer};
use crate::views::kit::{
    APP, Build, Data, WithData, all_sizes, app_series, missed, missed_metrics, response,
    response_metrics, wrap,
};

/// The capacity cards of a run, one per frame budget.
#[component]
pub fn CapacityCards(overview: Arc<RunOverview>) -> impl IntoView {
    let mut metrics = response_metrics();
    metrics.extend(missed_metrics());
    view! {
        <WithData overview=overview metrics=metrics skeletons=2 view=move |data| {
            BUDGETS.iter().map(|(hz, ms)| { let data = data.clone(); view! {
                <ChartCard
                    title=format!("How far each framework scales · within a {hz} Hz frame ({ms:.1} ms)")
                    note="Elements in the app up to which an interaction stays within the frame: the 95th percentile of the response time, and at most 1 % missed refreshes for scrolling and animation · darker is more · ≥: the largest size still held · <: the smallest size fails"
                    wide=true
                >
                    <CapacityMap data=data hz=*hz budget_ms=*ms/>
                </ChartCard>
            }}).collect_view()
        }/>
    }
}

/// One cell of the capacity table: the capacity, the points behind it and the cell to open.
struct Held<'a> {
    capacity: Option<Capacity>,
    points: Vec<(f64, f64)>,
    cell: Option<&'a CellInfo>,
    invalid: bool,
}

/// Returns the cell at which a capacity ends: the first size that fails, else the largest.
fn end_cell<'a>(cells: &[&'a CellInfo], points: &[(f64, f64)], limit: f64) -> Option<&'a CellInfo> {
    let failing = points.iter().find(|(_, v)| *v > limit).map(|(x, _)| *x);
    let at = failing.or_else(|| points.last().map(|(x, _)| *x))?;
    cells.iter().copied().find(|c| c.key.size() == at)
}

fn response_held<'a>(
    b: &Build<'a>,
    cells: &[&'a CellInfo],
    step: &str,
    limit: f64,
    open: bool,
) -> Held<'a> {
    let m = response(step);
    let measured: Vec<&CellInfo> = cells
        .iter()
        .copied()
        .filter(|c| !b.prefs.valid_only || c.is_valid(step))
        .collect();
    let points: Vec<(f64, f64)> = measured
        .iter()
        .filter_map(|c| {
            let stat = b.data.stat(&m, c.id)?;
            Some((c.key.size(), quantile(&sorted(&stat.values), 0.95)?))
        })
        .collect();
    Held {
        capacity: capacity(&points, limit, true, open),
        cell: end_cell(cells, &points, limit),
        invalid: measured
            .iter()
            .any(|c| b.data.stat(&m, c.id).is_some() && !c.is_valid(step)),
        points,
    }
}

/// Returns the capacity of a pacing measurement at a refresh rate. A size that misses refreshes
/// at the rate of every cell takes the share of its cell at the slower rate; without that cell
/// it fails.
fn paced_held<'a>(
    b: &Build<'a>,
    cells: &[&'a CellInfo],
    step: &str,
    hz: f64,
    main: f64,
    open: bool,
) -> Held<'a> {
    let m = missed(step);
    let limit = SMOOTH_SHARE * 100.0;
    let points: Vec<(f64, f64)> = cells
        .iter()
        .filter_map(|c| {
            let mut share = b.data.stat(&m, c.id)?.median;
            if share > limit && hz < main {
                share = b
                    .data
                    .overview()
                    .cells
                    .iter()
                    .find(|s| {
                        s.ran()
                            && s.key.scenario == c.key.scenario
                            && s.key.framework == c.key.framework
                            && s.key.rows == c.key.rows
                            && s.key.hz == hz
                    })
                    .and_then(|s| b.data.stat(&m, s.id))
                    .map_or(100.0, |s| s.median);
            }
            Some((c.key.size(), share))
        })
        .collect();
    Held {
        capacity: capacity(&points, limit, false, open),
        cell: end_cell(cells, &points, limit),
        invalid: false,
        points,
    }
}

#[component]
fn CapacityMap(data: Data, hz: u32, budget_ms: f64) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, Uncertainty::Auto);
        let main = data.overview().provenance.main_hz();
        let sizes = all_sizes(&b, APP);
        let smallest = sizes.first().map_or(1.0, |s| f64::from(s.1.unwrap_or(s.0)));
        let largest = sizes.last().map_or(1.0, |s| f64::from(s.1.unwrap_or(s.0)));
        let mut frameworks: Vec<String> = Vec::new();
        for c in b.cells(APP) {
            if !frameworks.contains(&c.key.framework) {
                frameworks.push(c.key.framework.clone());
            }
        }
        let cells = frameworks
            .iter()
            .map(|fw| {
                let series = app_series(&b, fw, main);
                let open = stop_of(data.overview(), APP, fw).is_none();
                let responses = RESPONSES.iter().map(|s| {
                    (
                        response_held(&b, &series, s.name, budget_ms, open),
                        "ms, 95th percentile",
                    )
                });
                let paced = PACED.iter().map(|s| {
                    (
                        paced_held(&b, &series, s.name, f64::from(hz), main, open),
                        "% of the refreshes missed",
                    )
                });
                responses
                    .chain(paced)
                    .map(|(held, words)| {
                        let mut tip: Vec<(String, String)> = held
                            .points
                            .iter()
                            .map(|(x, v)| {
                                (
                                    format!("{} elements", compact(*x)),
                                    format!("{} {words}", short(*v)),
                                )
                            })
                            .collect();
                        if !open {
                            tip.push((
                                "larger sizes".to_owned(),
                                "not run: over 250 ms before".to_owned(),
                            ));
                        }
                        Some(HeatCell {
                            text: held.capacity.map_or("–".to_owned(), Capacity::text),
                            t: held.capacity.map(|c| shade(c, smallest, largest)),
                            cell: held.cell.map(|c| c.id),
                            invalid: held.invalid,
                            tip,
                        })
                    })
                    .collect()
            })
            .collect();
        HeatModel {
            rows: frameworks
                .iter()
                .map(|f| (f.clone(), palette::ink(f)))
                .collect(),
            cols: RESPONSES
                .iter()
                .chain(&PACED)
                .map(|s| wrap(s.title, 14))
                .collect(),
            cells,
            diverging: false,
            ends: ("fewer elements".to_owned(), "more elements".to_owned()),
        }
    });
    view! { <Heatmap model=model/> }
}
