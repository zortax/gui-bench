//! The four metric cards: the label, the value, the delta pill and a sparkline of plain bars,
//! each bar an element with its height set inline. The value follows the live tick.

use vizia::prelude::*;

use super::live::Live;
use crate::dashboard_spec::{KPI_COUNT, KPIS, Kpi, kpi_value};

/// Builds the row of metric cards.
pub fn build(cx: &mut Context, live: Live) {
    HStack::new(cx, move |cx| {
        for index in 0..KPI_COUNT {
            card(cx, index, KPIS[index], live);
        }
    })
    .class("kpis");
}

/// Builds metric card `index`.
fn card(cx: &mut Context, index: usize, kpi: Kpi, live: Live) {
    VStack::new(cx, move |cx| {
        Label::new(cx, kpi.label).class("kpi-label");
        Label::new(cx, live.tick.map(move |tick| kpi_value(index, *tick))).class("kpi-value");
        Label::new(cx, kpi.delta)
            .class("delta")
            .toggle_class("up", kpi.up)
            .toggle_class("down", !kpi.up);
        HStack::new(cx, move |cx| {
            for height in kpi.spark {
                Element::new(cx).class("spark-bar").height(Pixels(height));
            }
        })
        .class("spark");
    })
    .class("card")
    .class("kpi");
}
