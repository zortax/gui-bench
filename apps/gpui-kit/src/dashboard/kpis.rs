//! The metric cards: their own view, which observes the live state and shows the value of its
//! tick. Each card holds a label, the value, a delta pill and a sparkline of plain bars, placed
//! at the card offsets of the spec.

use gpui_kit::component::h_flex;
use gpui_kit::*;

use super::live::Live;
use super::spec::{self, Kpi};
use super::style::card;

/// The row of the four metric cards.
pub struct Kpis {
    live: Entity<Live>,
    _live_changes: Subscription,
}

impl Kpis {
    pub fn new(live: Entity<Live>, cx: &mut Context<Self>) -> Self {
        let live_changes = cx.observe(&live, |_, _, cx| cx.notify());
        Self {
            live,
            _live_changes: live_changes,
        }
    }
}

impl Render for Kpis {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tick = self.live.read(cx).tick;
        h_flex()
            .flex_none()
            .w_full()
            .h(px(spec::KPI_HEIGHT))
            .gap(px(spec::GAP))
            .children(
                spec::KPIS
                    .iter()
                    .enumerate()
                    .map(|(k, kpi)| self::kpi(kpi, spec::kpi_value(k, tick))),
            )
    }
}

/// Returns one metric card that shows `value`.
fn kpi(kpi: &Kpi, value: String) -> Div {
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
                .child(value),
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
