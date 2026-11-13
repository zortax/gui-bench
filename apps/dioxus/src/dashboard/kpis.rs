//! The four metric cards: a label, a value, the delta pill and a sparkline of plain bars. The
//! value follows the live tick.

use dioxus::prelude::*;

use super::spec::{self, Kpi};

/// The row of metric cards, which share the width.
#[component]
pub fn Kpis(tick: ReadSignal<u64>) -> Element {
    rsx! {
        div { class: "kpis",
            for (index , kpi) in spec::KPIS.into_iter().enumerate() {
                KpiCard { key: "{index}", index, kpi, tick }
            }
        }
    }
}

/// Metric card `index`.
#[component]
fn KpiCard(index: usize, kpi: Kpi, tick: ReadSignal<u64>) -> Element {
    rsx! {
        div { class: "card kpi",
            div { class: "kpi__label", {kpi.label} }
            div { class: "kpi__value", {spec::kpi_value(index, tick())} }
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
