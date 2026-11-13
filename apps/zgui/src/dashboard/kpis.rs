//! The four metric cards: a label, a value, a delta pill and a sparkline each.

use zgui::prelude::*;

use super::live::Live;
use super::spec::{self, Kpi};

/// The row of metric cards, which share the width. The set of cards never changes, so a plain
/// list holds them; only the value text follows the live tick.
#[component]
pub fn MetricCards(live: Live) -> impl IntoView {
    view! {
        row(class = "kpis") {
            {(0..spec::KPI_COUNT)
                .map(|index| view! {
                    MetricCard(index = index, kpi = spec::KPIS[index], live = live)
                }.into_view())
                .collect::<Vec<_>>()}
        }
    }
}

/// Metric card `index`.
#[component]
fn MetricCard(index: usize, kpi: Kpi, live: Live) -> impl IntoView {
    view! {
        column(class = "card kpi") {
            text(class = "kpi__label") {{kpi.label}}
            text(class = "kpi__value") {{move || spec::kpi_value(index, live.tick.get())}}
            row(class = "delta", class:up = kpi.up, class:down = !kpi.up) {{kpi.delta}}
            row(class = "spark") {
                {Vec::from(kpi.spark.map(|height| view! {
                    box(class = "spark__bar", style:height = format!("{height}px"))
                }.into_view()))}
            }
        }
    }
}
