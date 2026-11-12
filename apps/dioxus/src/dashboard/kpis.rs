//! The four metric cards: a label, a value, the delta pill and a sparkline of plain bars.

use dioxus::prelude::*;

use super::spec::{self, Kpi};

/// The row of metric cards, which share the width.
#[component]
pub fn Kpis() -> Element {
    rsx! {
        div { class: "kpis",
            for (index , kpi) in spec::KPIS.into_iter().enumerate() {
                KpiCard { key: "{index}", kpi }
            }
        }
    }
}

/// One metric card.
#[component]
fn KpiCard(kpi: Kpi) -> Element {
    rsx! {
        div { class: "card kpi",
            div { class: "kpi__label", {kpi.label} }
            div { class: "kpi__value", {kpi.value} }
            span {
                class: "pill kpi__delta",
                class: if kpi.up { "up" } else { "down" },
                {kpi.delta}
            }
            div { class: "spark",
                for (index , height) in kpi.spark.into_iter().enumerate() {
                    div { key: "{index}", class: "spark__bar", height: "{height}px" }
                }
            }
        }
    }
}
