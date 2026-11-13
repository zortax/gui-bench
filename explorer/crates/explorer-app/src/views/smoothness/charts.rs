//! One pacing metric against the size of the app at a refresh rate: one line per framework
//! with the range of the iterations, and rules at the values that judge it.

use explorer_core::api::MetricRef;
use explorer_core::scale::ScaleKind;
use leptos::prelude::*;

use crate::chart::{Axis, ChartCard, Rule, XyModel, XyPlot, spread_note, use_card};
use crate::state::use_explorer;
use crate::views::kit::{APP, Build, Data, Scope, all_sizes, baseline_label, size_axis};

/// The rules of a pacing chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judge {
    /// One refresh at the rate of the chart; the value axis is logarithmic.
    Refresh,
    /// The share of missed refreshes up to which a measurement counts as smooth.
    Smooth,
    /// No rule.
    Count,
}

/// A card of one pacing metric against the size of the app.
#[component]
pub fn PacingCard(
    data: Data,
    #[prop(into)] title: String,
    metric: MetricRef,
    words: &'static str,
    judge: Judge,
    scope: Scope,
) -> impl IntoView {
    view! {
        <ChartCard title=title>
            <PacingPlot data=data metric=metric words=words judge=judge scope=scope/>
        </ChartCard>
    }
}

#[component]
fn PacingPlot(
    data: Data,
    metric: MetricRef,
    words: &'static str,
    judge: Judge,
    scope: Scope,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note.set(spread_note(false, ctx.uncertainty(), "Line"));
    });
    let unit = explorer_core::units::display(&data.unit(&metric))
        .label
        .to_owned();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let hz = scope
            .hz
            .get()
            .unwrap_or(data.overview().provenance.main_hz());
        let base = prefs.base.is_some();
        let mut y = Axis::value(
            if base {
                baseline_label(&prefs, "")
            } else {
                words.to_owned()
            },
            if base { "×".to_owned() } else { unit.clone() },
        );
        if base || judge == Judge::Refresh {
            y.kind = ScaleKind::Log;
        }
        let mut model = XyModel::new(size_axis(&all_sizes(&b, APP)), y);
        model.height = 260.0;
        model.series = b.size_series(APP, &metric, hz);
        let rule = |value: f64, label: String| Rule {
            horizontal: true,
            value,
            label,
        };
        match judge {
            _ if base => model.rules.push(rule(1.0, "baseline".to_owned())),
            Judge::Refresh => model
                .rules
                .push(rule(1000.0 / hz, format!("one refresh at {hz} Hz"))),
            Judge::Smooth => model
                .rules
                .push(rule(1.0, "1 %: smooth up to here".to_owned())),
            Judge::Count => {}
        }
        model
    });
    view! { <XyPlot model=model/> }
}
