//! The metric cards: label, value, delta pill and a sparkline of plain bars, placed at the card
//! offsets of the spec.

use gpui_kit::component::h_flex;
use gpui_kit::*;

use super::spec::{self, Kpi};
use super::style::card;

/// Returns the row of the four metric cards, which share the width.
pub fn kpis() -> Div {
    h_flex()
        .flex_none()
        .w_full()
        .h(px(spec::KPI_HEIGHT))
        .gap(px(spec::GAP))
        .children(spec::KPIS.iter().map(kpi))
}

/// Returns one metric card.
fn kpi(kpi: &Kpi) -> Div {
    let (background, text) = if kpi.up {
        (spec::UP_BG, spec::UP_TEXT)
    } else {
        (spec::DOWN_BG, spec::DOWN_TEXT)
    };
    card()
        .relative()
        .flex_1()
        .min_w_0()
        .h_full()
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .left(px(spec::CARD_PADDING))
                .top(px(18.0))
                .text_size(px(13.0))
                .text_color(rgb(spec::MUTED))
                .child(kpi.label),
        )
        .child(
            div()
                .absolute()
                .left(px(spec::CARD_PADDING))
                .top(px(40.0))
                .text_size(px(24.0))
                .font_weight(FontWeight::BOLD)
                .child(kpi.value),
        )
        .child(
            h_flex()
                .absolute()
                .left(px(spec::CARD_PADDING))
                .top(px(78.0))
                .h(px(22.0))
                .px(px(8.0))
                .rounded(px(11.0))
                .bg(rgb(background))
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(text))
                .child(kpi.delta),
        )
        .child(
            h_flex()
                .absolute()
                .right(px(spec::CARD_PADDING))
                .bottom(px(spec::KPI_HEIGHT - 92.0))
                .items_end()
                .gap(px(spec::SPARK_STEP - spec::SPARK_BAR_WIDTH))
                .children(kpi.spark.iter().map(|&height| {
                    div()
                        .flex_none()
                        .w(px(spec::SPARK_BAR_WIDTH))
                        .h(px(height))
                        .rounded(px(2.0))
                        .bg(rgb(spec::SPARK))
                })),
        )
}
