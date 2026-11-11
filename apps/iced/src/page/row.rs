//! A row of the header scenarios: its label on the left and its value on the right, above a 1 px
//! border. The row owns its text, so it lives as long as needed (`lazy` asks for `'static`).

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Container, column, container, row as line, space, text};
use iced::{Element, Length};

use super::color;
use crate::spec::{self, Row};

/// Returns a row with a label, a value and a background color.
pub fn row<Message: 'static>(
    label: &str,
    value: u32,
    background: u32,
) -> Container<'static, Message> {
    let content = line![
        text(label.to_owned())
            .size(spec::FONT_SIZE)
            .color(color(spec::LABEL_COLOR)),
        text(value)
            .size(spec::FONT_SIZE)
            .color(color(spec::VALUE_COLOR))
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(Vertical::Center)
    .padding([0.0, spec::PADDING_X])
    .width(Length::Fill)
    .height(spec::ROW_HEIGHT - spec::BORDER_HEIGHT);
    let border = container(space())
        .width(Length::Fill)
        .height(spec::BORDER_HEIGHT)
        .style(|_| container::background(color(spec::ROW_BORDER)));
    container(column![content, border])
        .width(Length::Fill)
        .style(move |_| container::background(color(background)))
}

/// Returns a static row on the plain row background.
pub fn plain_row<Message: 'static>(row: &Row) -> Element<'static, Message> {
    self::row(&row.label, row.value, spec::ROW_EVEN).into()
}
