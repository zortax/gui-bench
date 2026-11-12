//! The top bar: the page title, iced's text input for the search, the export button and the
//! avatar, above a 1 px border. The controls are centered on y 32 (the row has 1 px of top
//! padding to balance the border).

use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, space, text, text_input};
use iced::{Element, Length, padding};

use super::sidebar::avatar;
use super::style::{self, SEMIBOLD};
use super::{Message, spec};
use crate::page::color;

/// The space between the search input and the export button, and between the button and the
/// avatar.
const SEARCH_GAP: f32 = spec::EXPORT_X - spec::SEARCH_X - spec::SEARCH_WIDTH;
const AVATAR_GAP: f32 = spec::TOPBAR_AVATAR_X - spec::EXPORT_X - spec::EXPORT_WIDTH;
const AVATAR: f32 = 36.0;
/// The line height of the controls' text, so that the input is `CONTROL_HEIGHT` tall.
const LINE: f32 = 20.0;

/// Returns the top bar with the search text `search`.
pub fn view(search: &str) -> Element<'_, Message> {
    let search = text_input(spec::SEARCH_PLACEHOLDER, search)
        .on_input(Message::Search)
        .size(spec::FONT_SIZE)
        .line_height(text::LineHeight::Absolute(LINE.into()))
        .padding([(spec::CONTROL_HEIGHT - LINE) / 2.0, 12.0])
        .width(spec::SEARCH_WIDTH)
        .style(style::input);
    let export = button(
        text(spec::EXPORT_LABEL)
            .size(spec::FONT_SIZE)
            .font(SEMIBOLD)
            .center()
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(0)
    .width(spec::EXPORT_WIDTH)
    .height(spec::CONTROL_HEIGHT)
    .on_press(Message::Export)
    .style(style::primary);
    let content = row![
        text(spec::PAGE_TITLE)
            .size(spec::TITLE_FONT + 4.0)
            .font(SEMIBOLD)
            .color(color(spec::TEXT)),
        space::horizontal(),
        search,
        space().width(SEARCH_GAP),
        export,
        space().width(AVATAR_GAP),
        avatar(spec::USER_INITIALS, AVATAR, 13.0, spec::USER_COLOR),
    ]
    .align_y(Vertical::Center)
    .padding(padding::horizontal(spec::CONTENT_PADDING).top(1))
    .height(spec::TOPBAR_HEIGHT - 1.0);
    let border = container(space())
        .width(Length::Fill)
        .height(1)
        .style(|_| style::filled(spec::RULE, 0));
    container(column![content, border])
        .width(Length::Fill)
        .style(|_| style::filled(spec::TOPBAR_BG, 0))
        .into()
}
