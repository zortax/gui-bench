//! One row: a button that tracks hover by itself, so the hover color needs no message and no
//! view rebuild. The row holds a 31 px content line and a 1 px bottom border.

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row as line, text};
use iced::{Background, Border, Color, Element, Length, Shadow, color};

const ROW_HEIGHT: f32 = 32.0;
const BORDER_HEIGHT: f32 = 1.0;
const PADDING_X: f32 = 12.0;
const FONT_SIZE: f32 = 14.0;

pub const BACKGROUND: Color = color!(0xffffff);
pub const LABEL: Color = color!(0x1f2328);
const ROW_EVEN: Color = color!(0xffffff);
const ROW_ODD: Color = color!(0xf6f8fa);
const ROW_HOVER: Color = color!(0xcfe3ff);
const ROW_BORDER: Color = color!(0xe5e7eb);
const VALUE: Color = color!(0x6e7781);

/// The message of a row press.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    Pressed,
}

/// Builds row `i`.
pub fn row<'a>(i: usize) -> Element<'a, Message> {
    let base = if i.is_multiple_of(2) {
        ROW_EVEN
    } else {
        ROW_ODD
    };
    let content = line![
        text(format!("Row {i:05}")).size(FONT_SIZE).color(LABEL),
        text((i * 7919) % 10000)
            .size(FONT_SIZE)
            .color(VALUE)
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(Vertical::Center)
    .padding([0.0, PADDING_X])
    .width(Length::Fill)
    .height(ROW_HEIGHT - BORDER_HEIGHT);
    let border = container(column![])
        .width(Length::Fill)
        .height(BORDER_HEIGHT)
        .style(|_| container::Style::default().background(ROW_BORDER));
    button(column![content, border])
        .padding(0)
        .width(Length::Fill)
        .height(ROW_HEIGHT)
        .on_press(Message::Pressed)
        .style(move |_, status| {
            let background = match status {
                button::Status::Hovered | button::Status::Pressed => ROW_HOVER,
                button::Status::Active | button::Status::Disabled => base,
            };
            button::Style {
                background: Some(Background::Color(background)),
                text_color: LABEL,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            }
        })
        .into()
}
