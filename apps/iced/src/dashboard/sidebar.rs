//! The sidebar: the brand, the nav items as buttons whose style follows their hover status, and
//! the user at the bottom. It sits beside the scroll container, so it never scrolls.

use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, space, text};
use iced::{Background, Border, Element, Length, Shadow, padding};

use super::style::{self, SEMIBOLD, color};
use super::{Message, spec};

/// The top of the user block below its rule, and its left padding.
const USER_TOP: f32 = 19.0;
const SIDEBAR_PADDING: f32 = 20.0;
/// The space between an icon or avatar and its text.
const ICON_GAP: f32 = 12.0;

/// Returns the sidebar with nav item `active` active.
pub fn view<'a>(active: usize) -> Element<'a, Message> {
    let brand = row![
        container(space())
            .width(spec::LOGO_SIZE)
            .height(spec::LOGO_SIZE)
            .style(|_| style::filled(spec::LOGO, spec::LOGO_RADIUS)),
        text(spec::APP_NAME)
            .size(spec::TITLE_FONT)
            .font(SEMIBOLD)
            .color(color(0xffffff)),
    ]
    .spacing(ICON_GAP)
    .align_y(Vertical::Center)
    .padding([0.0, SIDEBAR_PADDING])
    .height(spec::TOPBAR_HEIGHT);
    let nav = column(
        spec::NAV_ITEMS
            .iter()
            .enumerate()
            .map(|(i, label)| item(i, label, i == active)),
    )
    .spacing(spec::NAV_STRIDE - spec::NAV_HEIGHT)
    .padding([0.0, spec::NAV_X]);
    let rule = container(space())
        .width(Length::Fill)
        .height(1)
        .style(|_| style::filled(spec::SIDEBAR_RULE, 0));
    let user = row![
        avatar(
            spec::USER_INITIALS,
            spec::USER_AVATAR,
            13.0,
            spec::USER_COLOR
        ),
        column![
            text(spec::USER_NAME)
                .size(spec::FONT_SIZE)
                .color(color(0xffffff)),
            text(spec::USER_ROLE)
                .size(12)
                .color(color(spec::SIDEBAR_MUTED)),
        ],
    ]
    .spacing(ICON_GAP)
    .align_y(Vertical::Center)
    .padding([0.0, SIDEBAR_PADDING]);
    let user = container(user)
        .height(spec::WINDOW.1 - spec::USER_RULE_Y - 1.0)
        .padding(padding::top(USER_TOP));
    container(column![
        brand,
        space().height(spec::NAV_TOP - spec::TOPBAR_HEIGHT),
        nav,
        space::vertical(),
        rule,
        user,
    ])
    .width(spec::SIDEBAR_WIDTH)
    .height(Length::Fill)
    .style(|_| style::filled(spec::SIDEBAR_BG, 0))
    .into()
}

/// Returns a circle of `size` with white initials.
pub fn avatar<'a, M: 'a>(
    initials: impl text::IntoFragment<'a>,
    size: f32,
    text_size: f32,
    background: u32,
) -> Element<'a, M> {
    container(
        text(initials)
            .size(text_size)
            .font(SEMIBOLD)
            .color(color(0xffffff)),
    )
    .center(size)
    .style(move |_| style::filled(background, size / 2.0))
    .into()
}

/// Returns nav item `i`: an icon square, the label and, on one item, the unread badge.
fn item<'a>(i: usize, label: &'a str, active: bool) -> Element<'a, Message> {
    let icon = if active {
        spec::NAV_ICON_ACTIVE
    } else {
        spec::NAV_ICON_COLOR
    };
    let mut content = row![
        container(space())
            .width(spec::NAV_ICON)
            .height(spec::NAV_ICON)
            .style(move |_| style::filled(icon, 4)),
        text(label).size(spec::FONT_SIZE),
    ]
    .spacing(ICON_GAP)
    .align_y(Vertical::Center)
    .padding(padding::left(16).right(ICON_GAP))
    .height(Length::Fill);
    if i == spec::BADGE_ITEM {
        let badge = container(
            text(spec::BADGE_COUNT)
                .size(12)
                .font(SEMIBOLD)
                .color(color(0xffffff)),
        )
        .center_y(20)
        .padding([0, 7])
        .style(|_| style::filled(spec::BADGE_BG, 10));
        content = content.push(space::horizontal()).push(badge);
    }
    button(content)
        .padding(0)
        .width(Length::Fill)
        .height(spec::NAV_HEIGHT)
        .on_press(Message::Nav(i))
        .style(move |_, status| {
            let background = match status {
                _ if active => Some(spec::NAV_ACTIVE),
                button::Status::Hovered | button::Status::Pressed => Some(spec::NAV_HOVER),
                button::Status::Active | button::Status::Disabled => None,
            };
            button::Style {
                background: background.map(|c| Background::Color(color(c))),
                text_color: color(if active { 0xffffff } else { spec::SIDEBAR_TEXT }),
                border: Border::default().rounded(spec::NAV_RADIUS),
                shadow: Shadow::default(),
                snap: true,
            }
        })
        .into()
}
