//! A frame metric against the size of a scenario at a refresh rate: one line per framework, in
//! milliseconds or in refreshes.

use explorer_core::scale::ScaleKind;
use leptos::prelude::*;

use super::source::Source;
use crate::chart::{Axis, ChartCard, Rule, XyModel, XyPlot, spread_note, use_card};
use crate::state::use_explorer;
use crate::views::kit::{Build, Data, Scope, all_sizes, baseline_label, size_axis};

/// A card of one frame metric against size.
#[component]
pub fn FrameScaling(
    data: Data,
    source: Source,
    metric: &'static str,
    #[prop(into)] title: String,
    words: &'static str,
    per_refresh: Signal<bool>,
    scope: Scope,
) -> impl IntoView {
    view! {
        <ChartCard title=title>
            <ScalingPlot data=data source=source metric=metric words=words per_refresh=per_refresh scope=scope/>
        </ChartCard>
    }
}

#[component]
fn ScalingPlot(
    data: Data,
    source: Source,
    metric: &'static str,
    words: &'static str,
    per_refresh: Signal<bool>,
    scope: Scope,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let m = source.metric(metric);
    let interval = metric.starts_with("interval");
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note.set(spread_note(false, ctx.uncertainty(), "Line"));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let rates: Vec<f64> = scope.hz.get().into_iter().collect();
        let base = prefs.base.is_some();
        let per = per_refresh.get() && interval && !base;
        let (label, unit) = if base {
            (baseline_label(&prefs, ""), "×".to_owned())
        } else if per {
            ("refreshes".to_owned(), "refreshes".to_owned())
        } else if interval {
            (words.to_owned(), "ms".to_owned())
        } else {
            (words.to_owned(), "%".to_owned())
        };
        let mut y = Axis::value(label, unit);
        if base || interval {
            y.kind = ScaleKind::Log;
        }
        let mut model = XyModel::new(size_axis(&all_sizes(&b, source.scenario)), y);
        model.height = 320.0;
        for hz in &rates {
            let factor = if per { hz / 1000.0 } else { 1.0 };
            model.series.extend(b.series_by(
                source.scenario,
                &m,
                *hz,
                |k| Some(k.size()),
                |_| factor,
            ));
            if interval && !base && !per {
                model.rules.push(Rule {
                    horizontal: true,
                    value: 1000.0 / hz,
                    label: format!("one refresh at {hz} Hz"),
                });
            }
        }
        if per {
            model.rules.push(Rule {
                horizontal: true,
                value: 1.0,
                label: "one refresh".to_owned(),
            });
        }
        if base {
            model.rules.push(Rule {
                horizontal: true,
                value: 1.0,
                label: "baseline".to_owned(),
            });
        }
        model
    });
    view! { <XyPlot model=model/> }
}
