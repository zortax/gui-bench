//! The metric cards: four cards that share the width, each with its label, value, delta pill and
//! a sparkline of plain rounded bars at the bottom right.

use floem::prelude::*;
use floem::taffy::AlignItems;

use super::style::{self, Weight, color, pill, px, text};
use crate::dash::{self, Kpi};

/// Returns the row of metric cards.
pub fn view() -> impl IntoView {
    Stack::horizontal_from_iter(dash::KPIS.iter().map(card)).style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(px(dash::KPI_HEIGHT))
            .col_gap(px(dash::GAP))
    })
}

/// One metric card: the texts on the left, the sparkline at the bottom right.
fn card(kpi: &Kpi) -> impl IntoView {
    let (pill_bg, pill_text) = if kpi.up {
        (dash::UP_BG, dash::UP_TEXT)
    } else {
        (dash::DOWN_BG, dash::DOWN_TEXT)
    };
    // Each text sits at the top of a band that reaches the next one: the label at card y 18,
    // the value at 40, the pill at 78.
    let label = band(text(kpi.label, 13.0, Weight::Normal, dash::MUTED), 22.0);
    let value = band(text(kpi.value, 24.0, Weight::Bold, dash::TEXT), 38.0);
    let delta = Container::new(text(kpi.delta, 12.0, Weight::Semibold, pill_text))
        .style(move |s| pill(s, 22.0, 8.0, pill_bg).align_self(AlignItems::FlexStart));
    let texts = Stack::vertical((label, value, delta)).style(|s| s.align_self(AlignItems::FlexStart));
    let spark = Stack::horizontal_from_iter(kpi.spark.iter().map(|&height| {
        Empty::new().style(move |s| {
            s.flex_shrink(0.0_f32)
                .size(px(dash::SPARK_BAR_WIDTH), px(height))
                .border_radius(2.0)
                .background(color(dash::SPARK))
        })
    }))
    .style(|s| {
        s.flex_shrink(0.0_f32)
            .col_gap(px(dash::SPARK_STEP - dash::SPARK_BAR_WIDTH))
            .items_end()
            .align_self(AlignItems::FlexEnd)
    });
    Stack::horizontal((texts, spark)).style(|s| {
        style::card(s)
            .flex_grow(1.0_f32)
            .flex_basis(0.0)
            .min_width(0.0)
            .height_full()
            .padding(px(dash::CARD_PADDING))
            .padding_top(18.0)
            .justify_between()
    })
}

/// Returns `label` at the top of a band `height` tall.
fn band(label: Label, height: f32) -> impl IntoView {
    Container::new(label).style(move |s| s.height(px(height)).items_start())
}
