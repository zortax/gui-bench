//! A frame metric against the size of a scenario: one line per framework and variant, dashed
//! per refresh rate, in milliseconds or in refreshes. Scenarios with many row counts get a log
//! axis of rows; others get one step per size.

use explorer_core::api::CellKey;
use explorer_core::scale::ScaleKind;
use leptos::prelude::*;

use super::source::Source;
use crate::chart::{Axis, ChartCard, Rule, XyModel, XyPlot, spread_note, use_card};
use crate::palette::hz_dash;
use crate::state::{Selection, use_explorer};
use crate::views::kit::{Build, Data, baseline_label, sizes_label};

/// How a chart reads the sizes of a scenario.
enum SizeAxis {
    Rows(Vec<f64>),
    Steps(Vec<(u32, Option<u32>)>),
}

impl SizeAxis {
    fn of(sizes: Vec<(u32, Option<u32>)>) -> Self {
        if sizes.len() >= 3 && sizes.iter().all(|(r, u)| *r > 0 && u.is_none()) {
            SizeAxis::Rows(sizes.iter().map(|(r, _)| f64::from(*r)).collect())
        } else {
            SizeAxis::Steps(sizes)
        }
    }

    fn axis(&self) -> Axis {
        match self {
            SizeAxis::Rows(rows) => Axis::log_fixed("rows", rows.clone()),
            SizeAxis::Steps(sizes) => Axis::steps(
                "size",
                sizes.iter().map(|(r, u)| sizes_label(*r, *u)).collect(),
            ),
        }
    }

    fn x(&self, k: &CellKey) -> Option<f64> {
        match self {
            SizeAxis::Rows(_) => Some(f64::from(k.rows)),
            SizeAxis::Steps(sizes) => sizes
                .iter()
                .position(|s| *s == (k.rows, k.updates))
                .map(|i| i as f64),
        }
    }
}

/// A card of one frame metric against size.
#[component]
pub fn FrameScaling(
    data: Data,
    source: Source,
    metric: &'static str,
    #[prop(into)] title: String,
    words: &'static str,
    per_refresh: Signal<bool>,
) -> impl IntoView {
    view! {
        <ChartCard title=title>
            <ScalingPlot data=data source=source metric=metric words=words per_refresh=per_refresh/>
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
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let m = source.metric(metric);
    let interval = metric.starts_with("interval");
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        let rates = explorer.prefs.rates.get();
        let dashes = if rates.len() == 1 {
            ""
        } else {
            " · dashed: the higher refresh rate"
        };
        ctx.note.set(format!(
            "{}{dashes}",
            spread_note(false, ctx.uncertainty(), "Line")
        ));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let sel = Selection::new(data.overview(), &prefs, source.scenario);
        let rates = sel.rates();
        let sizes = SizeAxis::of(sel.sizes());
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
        let mut model = XyModel::new(sizes.axis(), y);
        model.height = 320.0;
        for hz in &rates {
            let factor = if per { hz / 1000.0 } else { 1.0 };
            for mut s in b.series_by(source.scenario, &m, *hz, |k| sizes.x(k), |_| factor) {
                s.dash = hz_dash(*hz).to_owned();
                s.key = format!("{}@{hz}", s.key);
                if rates.len() > 1 {
                    s.label = format!("{} {hz} Hz", s.label);
                }
                s.end_label = rates.len() == 1 && !s.ghost;
                model.series.push(s);
            }
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
