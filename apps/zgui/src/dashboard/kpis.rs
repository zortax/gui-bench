//! The four metric cards: a label, a value, a delta pill and a sparkline each.

use zgui::prelude::*;

use super::spec::{self, Kpi};

/// The row of metric cards, which share the width. The cards never change, so a plain list holds
/// them.
#[component]
pub fn MetricCards() -> impl IntoView {
    view! {
        row(class = "kpis") {
            {Vec::from(spec::KPIS.map(|kpi| view! { MetricCard(kpi = kpi) }.into_view()))}
        }
    }
}

/// One metric card.
#[component]
fn MetricCard(kpi: Kpi) -> impl IntoView {
    view! {
        column(class = "card kpi") {
            text(class = "kpi__label") {{kpi.label}}
            text(class = "kpi__value") {{kpi.value}}
            row(class = "delta", class:up = kpi.up, class:down = !kpi.up) {{kpi.delta}}
            row(class = "spark") {
                {Vec::from(kpi.spark.map(|height| view! {
                    box(class = "spark__bar", style:height = format!("{height}px"))
                }.into_view()))}
            }
        }
    }
}
