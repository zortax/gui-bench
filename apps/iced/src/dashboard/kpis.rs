//! The metric cards: label, value and delta pill on the left, and a sparkline of plain rounded
//! containers in the bottom right corner, layered with a `stack`.

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{column, container, row, stack, text};
use iced::{Element, Length, padding};

use super::style::{self, BOLD, SEMIBOLD};
use super::{Message, spec};
use crate::page::color;

/// The top of the label, the value and the pill inside a card.
const LABEL_TOP: f32 = 18.0;
const VALUE_TOP: f32 = 40.0;
const PILL_TOP: f32 = 78.0;
const PILL_HEIGHT: f32 = 22.0;
/// The bottom of the sparkline bars inside a card.
const SPARK_BOTTOM: f32 = 92.0;

/// Returns the row of metric cards.
pub fn view<'a>() -> Element<'a, Message> {
    row(spec::KPIS.iter().map(card))
        .spacing(spec::GAP)
        .height(spec::KPI_HEIGHT)
        .into()
}

/// Returns one metric card.
fn card<'a>(kpi: &'a spec::Kpi) -> Element<'a, Message> {
    let (background, foreground) = if kpi.up {
        (spec::UP_BG, spec::UP_TEXT)
    } else {
        (spec::DOWN_BG, spec::DOWN_TEXT)
    };
    let pill = container(
        text(kpi.delta)
            .size(12)
            .font(SEMIBOLD)
            .color(color(foreground)),
    )
    .center_y(PILL_HEIGHT)
    .padding([0, 8])
    .style(move |_| style::filled(background, PILL_HEIGHT / 2.0));
    let numbers = column![
        container(text(kpi.label).size(13).color(color(spec::MUTED))).height(VALUE_TOP - LABEL_TOP),
        container(text(kpi.value).size(24).font(BOLD).color(color(spec::TEXT)))
            .height(PILL_TOP - VALUE_TOP),
        pill,
    ]
    .padding(padding::top(LABEL_TOP).left(spec::CARD_PADDING));
    let spark = row(kpi.spark.iter().map(|&height| {
        container(iced::widget::space())
            .width(spec::SPARK_BAR_WIDTH)
            .height(height)
            .style(|_| style::filled(spec::SPARK, 2))
            .into()
    }))
    .spacing(spec::SPARK_STEP - spec::SPARK_BAR_WIDTH)
    .align_y(Vertical::Bottom);
    let spark = container(spark)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Right)
        .align_y(Vertical::Bottom)
        .padding(padding::right(spec::CARD_PADDING).bottom(spec::KPI_HEIGHT - SPARK_BOTTOM));
    container(
        stack![numbers, spark]
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(style::card)
    .into()
}
