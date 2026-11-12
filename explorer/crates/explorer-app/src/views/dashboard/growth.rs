//! A metric at the smallest and the largest size of a scenario: per framework and variant, a
//! hollow marker at the small size and a dot with its interval at the large one, joined by a
//! line, so the growth with the data reads at a glance.

use explorer_core::api::{CellInfo, MetricRef};
use explorer_core::units::{display, short, thousands};
use leptos::prelude::*;

use crate::chart::{Axis, ChartCard, DotModel, DotPlot, DotRow, Rule, spread_note, use_card};
use crate::palette::Marker;
use crate::state::use_explorer;
use crate::views::kit::{Build, Data};

/// A card of one metric from the smallest to the largest size of a scenario at a refresh rate.
#[component]
pub fn GrowthCard(
    data: Data,
    scenario: &'static str,
    metric: MetricRef,
    #[prop(into)] title: String,
    words: &'static str,
    hz: Signal<Option<f64>>,
    /// Rules at one and two refreshes.
    #[prop(optional)]
    refresh_rules: bool,
) -> impl IntoView {
    view! {
        <ChartCard title=title>
            <GrowthPlot data=data scenario=scenario metric=metric words=words hz=hz refresh_rules=refresh_rules/>
        </ChartCard>
    }
}

#[component]
fn GrowthPlot(
    data: Data,
    scenario: &'static str,
    metric: MetricRef,
    words: &'static str,
    hz: Signal<Option<f64>>,
    refresh_rules: bool,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let samples = data.samples(&metric);
    let unit = display(&data.unit(&metric)).label.to_owned();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let mut cells: Vec<&CellInfo> = b.cells(scenario);
        // A scenario measured at one rate only shows that rate.
        let rates: Vec<f64> = cells.iter().map(|c| c.key.hz).collect();
        let want = hz
            .get()
            .filter(|h| rates.contains(h))
            .or_else(|| rates.first().copied());
        cells.retain(|c| Some(c.key.hz) == want);
        let small = cells.iter().map(|c| c.key.rows).min().unwrap_or(0);
        let large = cells.iter().map(|c| c.key.rows).max().unwrap_or(0);
        let mut rows: Vec<DotRow> = Vec::new();
        for c in cells.iter().filter(|c| c.key.rows == large) {
            let mut row = DotRow::of_framework(c.id.to_string(), &c.key.framework, c.key.label());
            row.marker = Marker::of_variant(&c.key.variant);
            row.cell = Some(c.id);
            let from_cell = cells.iter().find(|x| {
                x.key.rows == small
                    && x.key.framework == c.key.framework
                    && x.key.variant == c.key.variant
            });
            let from = from_cell.and_then(|x| b.value(&metric, x)).map(|v| v.y);
            if let Some(v) = b.value(&metric, c) {
                row.value = Some(v.y);
                row.band = v.spread.band;
                row.whisker = v.spread.whisker;
                row.invalid = v.invalid;
                row.tip = crate::views::kit::cell_tip(c, &v.stat, samples);
                if small != large {
                    row.from = from;
                    if let Some(f) = from {
                        row.text = format!("{} → {}", short(f), short(v.y));
                        row.tip.insert(
                            0,
                            (format!("{} rows", thousands(f64::from(small))), short(f)),
                        );
                        row.tip.insert(
                            1,
                            (format!("{} rows", thousands(f64::from(large))), short(v.y)),
                        );
                        if f > 0.0 {
                            row.tip
                                .insert(2, ("growth".to_owned(), format!("{}×", short(v.y / f))));
                        }
                    }
                }
            }
            row.ghost = None;
            rows.push(row);
        }
        let refresh = want.map_or(16.67, |h| 1000.0 / h);
        let mut x = Axis::value(words, unit.clone());
        x.zero = true;
        DotModel {
            rows,
            x,
            rules: if refresh_rules {
                [1.0, 2.0]
                    .iter()
                    .map(|k| Rule {
                        horizontal: false,
                        value: k * refresh,
                        label: format!("{k} refreshes"),
                    })
                    .collect()
            } else {
                Vec::new()
            },
            hide_labels: false,
            factors: false,
        }
    });
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        let rules = if refresh_rules {
            " · rules: one and two refreshes"
        } else {
            ""
        };
        ctx.note.set(format!(
            "{} · hollow: the smallest size{rules}",
            spread_note(samples, ctx.uncertainty(), "Dot at the largest size")
        ));
    });
    view! { <DotPlot model=model/> }
}
