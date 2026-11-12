//! The frame-time profile at one size and rate: per framework and variant, the median interval
//! as a dot, a band to p99, a whisker to the longest gap, and ticks at p90, p99 and p99.9. Rules
//! mark 1, 2, 4 and 8 refreshes.

use explorer_core::api::CellInfo;
use explorer_core::scale::ScaleKind;
use explorer_core::units::short;
use leptos::prelude::*;

use super::source::{LADDER, Source};
use crate::chart::{Axis, ChartCard, DotModel, DotPlot, DotRow, Rule, use_card};
use crate::palette::Marker;
use crate::state::use_explorer;
use crate::views::kit::{Build, Data, Scope};

/// A card of the frame-time profile.
#[component]
pub fn ProfileCard(
    data: Data,
    source: Source,
    scope: Scope,
    per_refresh: Signal<bool>,
    #[prop(optional, into)] title: Option<String>,
) -> impl IntoView {
    view! {
        <ChartCard
            title=title.unwrap_or_else(|| "Frame-time profile".to_owned())
            note="Dot: median interval · band: to p99 · whisker: to the longest gap · ticks: p90, p99, p99.9 · labels: p50 → p99 → longest · medians of iterations · rules: 1, 2, 4 and 8 refreshes"
        >
            <ProfilePlot data=data source=source scope=scope per_refresh=per_refresh/>
        </ChartCard>
    }
}

fn row_of(b: &Build, source: &Source, c: &CellInfo, per: f64, many_rates: bool) -> DotRow {
    let mut label = c.key.label();
    if many_rates {
        label.push_str(&format!(" {} Hz", c.key.hz));
    }
    let mut row = DotRow::of_framework(c.id.to_string(), &c.key.framework, label);
    row.marker = Marker::of_variant(&c.key.variant);
    row.hollow = c.key.variant == "coarse";
    row.cell = Some(c.id);
    let at = |m: &str| b.data.stat(&source.metric(m), c.id).map(|s| s.median * per);
    let Some(p50) = at("interval_p50") else {
        return row;
    };
    let p99 = at("interval_p99");
    let max = at("interval_max");
    row.value = Some(p50);
    row.band = p99.map(|v| (p50, v));
    row.whisker = max.map(|v| (p50, v));
    row.invalid = !c.is_valid(source.measurement);
    row.ticks = LADDER[1..5]
        .iter()
        .filter(|(m, _)| *m != "interval_p95")
        .filter_map(|(m, name)| Some((at(m)?, (*name).to_owned())))
        .collect();
    if let Some(v) = max {
        row.tip.push(("max".to_owned(), short(v)));
    }
    if let Some(s) = b.data.stat(&source.metric("stale_refreshes"), c.id) {
        row.tip.push((
            "stale refreshes".to_owned(),
            format!("{}%", short(s.median)),
        ));
    }
    let mut text: Vec<String> = Vec::new();
    for v in [Some(p50), p99, max].iter().flatten() {
        let t = short(*v);
        if text.last() != Some(&t) {
            text.push(t);
        }
    }
    row.text = text.join(" → ");
    row
}

#[component]
fn ProfilePlot(
    data: Data,
    source: Source,
    scope: Scope,
    per_refresh: Signal<bool>,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let hz = scope.hz.get().unwrap_or(60.0);
        let per_on = per_refresh.get();
        let per = if per_on { hz / 1000.0 } else { 1.0 };
        let cells: Vec<&CellInfo> = b
            .cells(source.scenario)
            .into_iter()
            .filter(|c| scope.holds(c.key.rows, c.key.updates, c.key.hz))
            .filter(|c| !prefs.valid_only || c.is_valid(source.measurement))
            .collect();
        let rows = cells
            .iter()
            .map(|c| row_of(&b, &source, c, per, false))
            .collect();
        let refresh = if per_on { 1.0 } else { 1000.0 / hz };
        let mut x = Axis::value(
            if per_on {
                "refreshes"
            } else {
                "milliseconds per frame"
            },
            if per_on { "refreshes" } else { "ms" },
        );
        x.kind = ScaleKind::Log;
        DotModel {
            rows,
            x,
            rules: [1.0, 2.0, 4.0, 8.0]
                .iter()
                .map(|k| Rule {
                    horizontal: false,
                    value: k * refresh,
                    label: format!("{k} refreshes"),
                })
                .collect(),
            hide_labels: false,
            factors: false,
        }
    });
    view! { <DotPlot model=model/> }
}
