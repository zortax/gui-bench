//! The response time of one interaction against the size of the app: one line per framework
//! with the spread of the repeats, the frame budgets as rules, and a cross at the first size
//! that a framework did not run.

use explorer_core::scale::ScaleKind;
use explorer_core::story::{BUDGETS, RESPONSES, Step};
use leptos::prelude::*;

use crate::chart::{Axis, ChartCard, Pt, Rule, Series, XyModel, XyPlot, spread_note, use_card};
use crate::palette::Marker;
use crate::state::{PrefsSnapshot, Uncertainty, stop_of, use_explorer};
use crate::views::kit::{APP, Build, Data, all_sizes, baseline_label, response, size_axis};

/// The lower end of the shared value axis in milliseconds.
const FLOOR_MS: f64 = 0.2;

/// Returns the largest 95th percentile of every response of the shown cells, which sets the
/// shared value axis. `None` with a baseline, since ratios need no shared scale.
pub fn shared_top(data: &Data, prefs: &PrefsSnapshot) -> Option<f64> {
    if prefs.base.is_some() {
        return None;
    }
    let b = Build::new(data, prefs, Uncertainty::P90);
    let main = data.overview().provenance.main_hz();
    let cells: Vec<_> = b
        .cells(APP)
        .into_iter()
        .filter(|c| c.key.hz == main)
        .collect();
    RESPONSES
        .iter()
        .flat_map(|s| {
            let m = response(s.name);
            cells
                .iter()
                .filter_map(|c| {
                    let v = b.value(&m, c)?;
                    Some(v.spread.whisker.map_or(v.y, |(_, hi)| hi))
                })
                .collect::<Vec<_>>()
        })
        .max_by(f64::total_cmp)
}

/// A card of the response time of one interaction against the size of the app.
#[component]
pub fn ResponseCard(data: Data, step: Step, top: Signal<Option<f64>>) -> impl IntoView {
    view! {
        <ChartCard title=step.title>
            <ResponsePlot data=data step=step top=top/>
        </ChartCard>
    }
}

#[component]
fn ResponsePlot(data: Data, step: Step, top: Signal<Option<f64>>) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let m = response(step.name);
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note.set(spread_note(true, ctx.uncertainty(), "Line"));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let overview = data.overview();
        let main = overview.provenance.main_hz();
        let base = prefs.base.is_some();
        let mut y = Axis::value(
            if base {
                baseline_label(&prefs, "ms")
            } else {
                "ms, input to finished frame".to_owned()
            },
            if base { "×" } else { "ms" },
        );
        y.kind = ScaleKind::Log;
        if let Some(top) = top.get().filter(|_| !base) {
            y.domain = Some((FLOOR_MS, top.max(1.0) * 1.6));
        }
        let mut model = XyModel::new(size_axis(&all_sizes(&b, APP)), y);
        model.height = 250.0;
        model.series = b.size_series(APP, &m, main);
        let stops: Vec<Series> = model
            .series
            .iter()
            .filter(|s| !s.ghost)
            .filter_map(|s| {
                let at = stop_of(overview, APP, &s.group)?;
                let last = s.points.last()?;
                let mut stop =
                    Series::of_framework(format!("stop/{}", s.group), &s.group, s.label.clone());
                stop.marker = Marker::Cross;
                stop.line = false;
                stop.radius = 4.0;
                let mut p = Pt::new(at, last.y);
                p.cell = overview
                    .cells
                    .iter()
                    .find(|c| {
                        !c.ran()
                            && c.key.scenario == APP
                            && c.key.framework == s.group
                            && c.key.size() == at
                    })
                    .map(|c| c.id);
                p.tip = vec![(
                    "not run".to_owned(),
                    "the hover response was over 250 ms at a smaller size".to_owned(),
                )];
                stop.points.push(p);
                Some(stop)
            })
            .collect();
        model.series.extend(stops);
        if base {
            model.rules.push(Rule {
                horizontal: true,
                value: 1.0,
                label: "baseline".to_owned(),
            });
        } else {
            model.rules.extend(BUDGETS.iter().map(|(hz, ms)| Rule {
                horizontal: true,
                value: *ms,
                label: format!("{hz} Hz frame"),
            }));
        }
        model
    });
    view! { <XyPlot model=model/> }
}
