//! A dot plot card of one metric over the shown cells of one or more scenarios at a picked size
//! and refresh rate, or at the size and rate of a scope, and the picker of that size and rate.

use explorer_core::api::MetricRef;
use explorer_core::scale::ScaleKind;
use leptos::prelude::*;

use super::build::{Build, baseline_label};
use super::controls::{SizePicker, sizes_label};
use super::data::Data;
use super::scope::Scope;
use crate::chart::{Axis, ChartCard, DotModel, DotPlot, DotRow, Rule, spread_note, use_card};
use crate::state::{Selection, use_explorer};

/// Parses a size key `rows:updates`.
pub fn parse_size(key: &str) -> (u32, u32) {
    key.split_once(':').map_or((0, 0), |(r, u)| {
        (r.parse().unwrap_or(0), u.parse().unwrap_or(0))
    })
}

/// A picker of the size and the refresh rate of the shown cells of scenarios.
#[component]
pub fn ScopePicker(
    data: Data,
    scenarios: Vec<&'static str>,
    size: RwSignal<String>,
    rate: RwSignal<String>,
) -> impl IntoView {
    let explorer = use_explorer();
    let (d, s) = (data.clone(), scenarios.clone());
    let sizes = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let mut out: Vec<(String, String)> = Vec::new();
        for sc in &s {
            for (r, u) in Selection::new(d.overview(), &prefs, sc).sizes() {
                let item = (format!("{r}:{}", u.unwrap_or(0)), sizes_label(r, u));
                if !out.contains(&item) {
                    out.push(item);
                }
            }
        }
        out.sort_by_key(|(k, _)| parse_size(k));
        out
    });
    let rates = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let mut hz: Vec<f64> = Vec::new();
        for sc in &scenarios {
            for r in Selection::new(data.overview(), &prefs, sc).rates() {
                if !hz.contains(&r) {
                    hz.push(r);
                }
            }
        }
        hz.sort_by(f64::total_cmp);
        hz.into_iter()
            .map(|h| (h.to_string(), format!("{h} Hz")))
            .collect::<Vec<_>>()
    });
    view! {
        <div class="gx-view__controls">
            <SizePicker options=sizes value=size title="Size"/>
            <SizePicker options=rates value=rate title="Refresh rate"/>
        </div>
    }
}

/// A card with a dot plot of one metric.
#[component]
pub fn DotsCard(
    data: Data,
    #[prop(into)] title: String,
    words: &'static str,
    metric: MetricRef,
    /// The scenarios with their section headings; one scenario shows no heading.
    scenarios: Vec<(&'static str, &'static str)>,
    size: RwSignal<String>,
    rate: RwSignal<String>,
) -> impl IntoView {
    let at = Signal::derive(move || {
        let (rows, updates) = parse_size(&size.get());
        let hz: f64 = rate.get().parse().unwrap_or(0.0);
        Some((rows, (updates > 0).then_some(updates), hz))
    });
    let scenarios: Vec<(String, String)> = scenarios
        .iter()
        .map(|(s, h)| ((*s).to_owned(), (*h).to_owned()))
        .collect();
    view! {
        <ChartCard title=title>
            <Dots data=data words=words metric=metric scenarios=Signal::stored(scenarios) at=at/>
        </ChartCard>
    }
}

/// A card with a dot plot of one metric at the size and rate of a scope.
#[component]
pub fn ScopeDotsCard(
    data: Data,
    #[prop(into)] title: String,
    words: &'static str,
    metric: MetricRef,
    #[prop(into)] scenario: Signal<String>,
    scope: Scope,
) -> impl IntoView {
    let at = Signal::derive(move || {
        let (rows, updates) = scope.size.get()?;
        Some((rows, updates, scope.hz.get()?))
    });
    let scenarios = Signal::derive(move || vec![(scenario.get(), String::new())]);
    view! {
        <ChartCard title=title>
            <Dots data=data words=words metric=metric scenarios=scenarios at=at/>
        </ChartCard>
    }
}

#[component]
fn Dots(
    data: Data,
    words: &'static str,
    metric: MetricRef,
    #[prop(into)] scenarios: Signal<Vec<(String, String)>>,
    /// The rows, ticker updates and rate of the shown cells.
    at: Signal<Option<(u32, Option<u32>, f64)>>,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let samples = data.samples(&metric);
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note.set(spread_note(samples, ctx.uncertainty(), "Dot"));
    });
    let unit = explorer_core::units::display(&data.unit(&metric))
        .label
        .to_owned();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let at = at.get();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let scenarios = scenarios.get();
        let mut out: Vec<DotRow> = Vec::new();
        for (sc, heading) in &scenarios {
            let r = b.dot_rows(sc, &metric, |k| {
                at.is_some_and(|(rows, updates, hz)| {
                    k.rows == rows && k.updates.unwrap_or(0) == updates.unwrap_or(0) && k.hz == hz
                })
            });
            if scenarios.len() > 1 && !r.is_empty() {
                out.push(DotRow::header(heading.clone()));
            }
            out.extend(r);
        }
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
            Vec::new()
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
