//! The header: the status text on the left and one element on the right, above a 1 px border.

use iced::alignment::Vertical;
use iced::widget::{column, container, row, space, text};
use iced::{Element, Length};

use super::color;
use crate::spec;

/// Returns the 40 px header that shows `status` and `right`.
pub fn header<'a, Message: 'a>(
    status: String,
    right: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let content = row![
        text(status)
            .size(spec::FONT_SIZE)
            .color(color(spec::LABEL_COLOR)),
        space::horizontal(),
        right.into(),
    ]
    .align_y(Vertical::Center)
    .padding([0.0, spec::PADDING_X])
    .width(Length::Fill)
    .height(spec::HEADER_HEIGHT - spec::BORDER_HEIGHT);
    let border = container(space())
        .width(Length::Fill)
        .height(spec::BORDER_HEIGHT)
        .style(|_| container::background(color(spec::HEADER_BORDER)));
    container(column![content, border])
        .width(Length::Fill)
        .style(|_| container::background(color(spec::HEADER_BG)))
        .into()
}
