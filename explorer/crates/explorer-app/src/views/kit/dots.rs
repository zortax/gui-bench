//! A dot plot card of one metric over the shown cells of a scenario at the refresh rate of a
//! scope: at the size of the scope, or at every size.

use explorer_core::api::MetricRef;
use explorer_core::scale::ScaleKind;
use leptos::prelude::*;

use super::build::{Build, baseline_label};
use super::data::Data;
use super::scope::Scope;
use crate::chart::{Axis, ChartCard, DotModel, DotPlot, Rule, spread_note, use_card};
use crate::state::use_explorer;

/// A card with a dot plot of one metric.
#[component]
pub fn ScopeDotsCard(
    data: Data,
    #[prop(into)] title: String,
    words: &'static str,
    metric: MetricRef,
    scenario: &'static str,
    scope: Scope,
    /// Show the cells of every size at the rate of the scope.
    #[prop(optional)]
    every_size: bool,
    /// Rules at values of the axis, with their labels.
    #[prop(optional)]
    rules: Vec<(f64, String)>,
) -> impl IntoView {
    view! {
        <ChartCard title=title>
            <Dots data=data words=words metric=metric scenario=scenario scope=scope every_size=every_size rules=rules/>
        </ChartCard>
    }
}

#[component]
fn Dots(
    data: Data,
    words: &'static str,
    metric: MetricRef,
    scenario: &'static str,
    scope: Scope,
    every_size: bool,
    rules: Vec<(f64, String)>,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let samples = data.samples(&metric);
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note.set(format!(
            "{} · hollow: a part built by hand",
            spread_note(samples, ctx.uncertainty(), "Dot")
        ));
    });
    let unit = explorer_core::units::display(&data.unit(&metric))
        .label
        .to_owned();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let hz = scope.hz.get();
        let rows = scope.size.get().map(|s| s.0);
        let out = b.dot_rows(scenario, &metric, |k| {
            Some(k.hz) == hz && (every_size || Some(k.rows) == rows)
        });
        let base = prefs.base.is_some();
        let mut x = Axis::value(
            if base {
                baseline_label(&prefs, "")
            } else {
                words.to_owned()
            },
            if base { "×".to_owned() } else { unit.clone() },
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
            rules
                .iter()
                .map(|(value, label)| Rule {
                    horizontal: false,
                    value: *value,
                    label: label.clone(),
                })
                .collect()
        };
        DotModel {
            rows: out,
            x,
            rules,
            hide_labels: false,
            factors: false,
        }
    });
    view! { <DotPlot model=model/> }
}
