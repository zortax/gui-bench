//! The filter card: iced's checkboxes for the statuses, iced's toggler as the compare switch, and
//! the Reset and Apply buttons.

use iced::alignment::Vertical;
use iced::widget::{button, checkbox, column, container, row, space, text, toggler};
use iced::{Background, Border, Element, Length};

use super::style::{self, SEMIBOLD};
use super::{Message, spec};
use crate::page::color;

/// The top of the card in the window, and the heights of the title line, the compare row and the
/// space above and below it.
const ROW2_TOP: f32 = spec::TOPBAR_HEIGHT + spec::CONTENT_PADDING + spec::KPI_HEIGHT + spec::GAP;
const TITLE_HEIGHT: f32 = spec::CHECK_TOP - ROW2_TOP - spec::CARD_PADDING;
const COMPARE_HEIGHT: f32 = 28.0;
const COMPARE_GAP: f32 = spec::COMPARE_Y - (spec::CHECK_TOP + 4.0 * spec::CHECK_STRIDE);
const BUTTONS_GAP: f32 = spec::BUTTONS_Y - spec::COMPARE_Y - COMPARE_HEIGHT;
/// The space between a checkbox and its label.
const CHECK_GAP: f32 = 10.0;

/// Returns the filter card with the status checkboxes `shown` and the compare switch.
pub fn view<'a>(shown: &[bool; 4], compare: bool) -> Element<'a, Message> {
    let title = container(
        text(spec::FILTERS_TITLE)
            .size(spec::TITLE_FONT)
            .font(SEMIBOLD)
            .color(color(spec::TEXT)),
    )
    .height(TITLE_HEIGHT);
    let checks = column(spec::Status::ALL.iter().map(|&status| {
        let check = checkbox(shown[status.filter_row()])
            .label(status.label())
            .on_toggle(move |on| Message::Shown(status, on))
            .size(spec::CHECK_SIZE)
            .spacing(CHECK_GAP)
            .text_size(spec::FONT_SIZE)
            .style(check_style);
        container(check).center_y(spec::CHECK_STRIDE).into()
    }));
    let switch = toggler(compare)
        .on_toggle(Message::Compare)
        .size(spec::SWITCH_HEIGHT)
        .style(switch_style);
    let compare = row![
        text(spec::COMPARE_LABEL)
            .size(spec::FONT_SIZE)
            .color(color(spec::TEXT)),
        space::horizontal(),
        switch,
    ]
    .align_y(Vertical::Center)
    .height(COMPARE_HEIGHT);
    let buttons = row![
        space::horizontal(),
        action(spec::RESET_LABEL, Message::Reset, style::secondary),
        action(spec::APPLY_LABEL, Message::Apply, style::primary),
    ]
    .spacing(spec::APPLY_X - spec::RESET_X - spec::BUTTON_WIDTH);
    container(column![
        title,
        checks,
        space().height(COMPARE_GAP),
        compare,
        space().height(BUTTONS_GAP),
        buttons,
    ])
    .padding(spec::CARD_PADDING)
    .width(spec::FILTERS_WIDTH)
    .height(Length::Fill)
    .style(style::card)
    .into()
}

/// Returns a button of the filter card.
fn action<'a>(
    label: &'a str,
    message: Message,
    style: fn(&iced::Theme, button::Status) -> button::Style,
) -> Element<'a, Message> {
    button(
        text(label)
            .size(spec::FONT_SIZE)
            .font(SEMIBOLD)
            .center()
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(0)
    .width(spec::BUTTON_WIDTH)
    .height(spec::CONTROL_HEIGHT)
    .on_press(message)
    .style(style)
    .into()
}

/// A checked box is primary with a white check; an unchecked one white with a gray border.
fn check_style(_: &iced::Theme, status: checkbox::Status) -> checkbox::Style {
    let checked = match status {
        checkbox::Status::Active { is_checked }
        | checkbox::Status::Hovered { is_checked }
        | checkbox::Status::Disabled { is_checked } => is_checked,
    };
    let (background, border) = if checked {
        (spec::PRIMARY, spec::PRIMARY)
    } else {
        (spec::CARD_BG, spec::INPUT_BORDER)
    };
    checkbox::Style {
        background: Background::Color(color(background)),
        icon_color: color(spec::PRIMARY_TEXT),
        border: Border {
            color: color(border),
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(color(spec::TEXT)),
    }
}

/// The switch: a gray track when off, the primary color when on, and a white knob 2 px inside.
fn switch_style(_: &iced::Theme, status: toggler::Status) -> toggler::Style {
    let on = match status {
        toggler::Status::Active { is_toggled }
        | toggler::Status::Hovered { is_toggled }
        | toggler::Status::Disabled { is_toggled } => is_toggled,
    };
    toggler::Style {
        background: Background::Color(color(if on { spec::PRIMARY } else { spec::SWITCH_OFF })),
        background_border_width: 0.0,
        background_border_color: color(spec::SWITCH_OFF),
        foreground: Background::Color(color(0xffffff)),
        foreground_border_width: 0.0,
        foreground_border_color: color(0xffffff),
        text_color: None,
        border_radius: None,
        padding_ratio: 2.0 / spec::SWITCH_HEIGHT,
    }
}
