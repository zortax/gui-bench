//! The four metric cards: the label, the value, the delta pill and a sparkline of plain bars,
//! each bar an element with its height set inline.

use vizia::prelude::*;

use crate::dashboard_spec::{KPIS, Kpi};

/// Builds the row of metric cards.
pub fn build(cx: &mut Context) {
    HStack::new(cx, |cx| {
        for kpi in KPIS {
            card(cx, kpi);
        }
    })
    .class("kpis");
}

/// Builds one metric card.
fn card(cx: &mut Context, kpi: Kpi) {
    VStack::new(cx, move |cx| {
        Label::new(cx, kpi.label).class("kpi-label");
        Label::new(cx, kpi.value).class("kpi-value");
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
