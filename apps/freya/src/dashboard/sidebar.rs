//! The sidebar: the brand, the nav items and the user at the bottom. The nav items are freya's
//! `SideBarItem`s inside `Activable`, which tells the item whether it is the active one; the item
//! keeps its hover state itself and shows its hover color at once.

use freya::prelude::*;

use super::{avatar, color, spec, text};

/// The sidebar. It reads the active item, so a nav click renders the sidebar alone.
#[derive(PartialEq)]
pub struct Sidebar {
    pub active: State<usize>,
}

impl Component for Sidebar {
    fn render(&self) -> impl IntoElement {
        let active = self.active;
        let current = *active.read();
        let items = spec::NAV_ITEMS.iter().enumerate().map(|(i, name)| {
            Activable::new(nav_item(i, name, i == current, active))
                .active(i == current)
                .key(i)
        });

        rect()
            .width(Size::px(spec::SIDEBAR_WIDTH))
            .height(Size::fill())
            .background(color(spec::SIDEBAR_BG))
            .main_align(Alignment::SpaceBetween)
            .child(
                rect().width(Size::fill()).child(brand()).child(
                    rect()
                        .width(Size::fill())
                        .padding(Gaps::new(
                            spec::NAV_TOP - spec::TOPBAR_HEIGHT,
                            spec::NAV_X,
                            0.0,
                            spec::NAV_X,
                        ))
                        .spacing(spec::NAV_STRIDE - spec::NAV_HEIGHT)
                        .children(items),
                ),
            )
            .child(user())
    }
}

/// Returns the logo and the app name, centered on y 32.
fn brand() -> Rect {
    rect()
        .horizontal()
        .width(Size::fill())
        .height(Size::px(spec::TOPBAR_HEIGHT))
        .padding(Gaps::new(0.0, 0.0, 0.0, 20.0))
        .spacing(12.0)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .width(Size::px(spec::LOGO_SIZE))
                .height(Size::px(spec::LOGO_SIZE))
                .corner_radius(spec::LOGO_RADIUS)
                .background(color(spec::LOGO)),
        )
        .child(text(
            spec::APP_NAME,
            spec::TITLE_FONT,
            FontWeight::SEMI_BOLD,
            0xffffff,
        ))
}

/// Returns nav item `i`: the icon, the label and, on the badge item, the unread count.
/// A press makes it the active item.
fn nav_item(i: usize, name: &'static str, active: bool, mut current: State<usize>) -> SideBarItem {
    let (icon, label) = if active {
        (spec::NAV_ICON_ACTIVE, 0xffffff)
    } else {
        (spec::NAV_ICON_COLOR, spec::SIDEBAR_TEXT)
    };
    let theme = SideBarItemThemePartial::new()
        .color(color(label))
        .background(Color::TRANSPARENT)
        .hover_background(color(spec::NAV_HOVER))
        .active_background(color(spec::NAV_ACTIVE))
        .focus_border_fill(color(spec::NAV_ICON_ACTIVE))
        .corner_radius(spec::NAV_RADIUS)
        .margin(Gaps::new_all(0.0))
        .padding(Gaps::new_all(0.0));
    let badge = (i == spec::BADGE_ITEM).then(|| {
        super::pill(
            spec::BADGE_COUNT.to_string(),
            20.0,
            7.0,
            (spec::BADGE_BG, 0xffffff),
        )
    });

    SideBarItem::new()
        .theme(theme)
        .on_press(move |_| current.set(i))
        .child(
            rect()
                .horizontal()
                .content(Content::Flex)
                .width(Size::fill())
                .height(Size::px(spec::NAV_HEIGHT))
                .padding(Gaps::new(0.0, 12.0, 0.0, 16.0))
                .spacing(12.0)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .width(Size::px(spec::NAV_ICON))
                        .height(Size::px(spec::NAV_ICON))
                        .corner_radius(4.0)
                        .background(color(icon)),
                )
                .child(
                    text(name, spec::FONT_SIZE, FontWeight::NORMAL, label).width(Size::flex(1.0)),
                )
                .maybe_child(badge),
        )
}

/// Returns the user block: the rule at y 720, the avatar, the name and the role.
fn user() -> Rect {
    rect()
        .horizontal()
        .width(Size::fill())
        .height(Size::px(spec::WINDOW.1 - spec::USER_RULE_Y))
        .padding(Gaps::new(20.0, 0.0, 0.0, 20.0))
        .spacing(12.0)
        .border(
            Border::new()
                .fill(color(spec::SIDEBAR_RULE))
                .width(BorderWidth {
                    top: 1.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .alignment(BorderAlignment::Inner),
        )
        .child(avatar(
            spec::USER_INITIALS,
            spec::USER_AVATAR,
            spec::USER_COLOR,
            12.0,
        ))
        .child(
            rect()
                .height(Size::px(spec::USER_AVATAR))
                .main_align(Alignment::Center)
                .child(text(
                    spec::USER_NAME,
                    spec::FONT_SIZE,
                    FontWeight::NORMAL,
                    0xffffff,
                ))
                .child(text(
                    spec::USER_ROLE,
                    12.0,
                    FontWeight::NORMAL,
                    spec::SIDEBAR_MUTED,
                )),
        )
}
