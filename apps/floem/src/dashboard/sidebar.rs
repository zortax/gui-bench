//! The sidebar: the brand, the nav items and the user block at the bottom. A nav item takes the
//! hover color at once while the pointer is over it, and a click makes it the active one.

use floem::prelude::*;

use super::State;
use super::style::{Weight, circle, color, pill, px, text};
use crate::dash;

const WHITE: u32 = 0xffffff;

/// Returns the sidebar.
pub fn view(state: State) -> impl IntoView {
    let nav = Stack::vertical_from_iter(
        dash::NAV_ITEMS
            .iter()
            .enumerate()
            .map(|(i, label)| nav_item(i, label, state)),
    )
    .style(|s| {
        s.width_full()
            .margin_top(px(dash::NAV_TOP - dash::TOPBAR_HEIGHT))
            .padding_horiz(px(dash::NAV_X))
            .row_gap(px(dash::NAV_STRIDE - dash::NAV_HEIGHT))
    });
    let spacer = Empty::new().style(|s| s.flex_grow(1.0_f32));
    Stack::vertical((brand(), nav, spacer, user())).style(|s| {
        s.flex_shrink(0.0_f32)
            .width(px(dash::SIDEBAR_WIDTH))
            .height_full()
            .background(color(dash::SIDEBAR_BG))
    })
}

/// The logo square and the app name, centered in the top 64 px.
fn brand() -> impl IntoView {
    let logo = Empty::new().style(|s| {
        s.size(px(dash::LOGO_SIZE), px(dash::LOGO_SIZE))
            .border_radius(px(dash::LOGO_RADIUS))
            .background(color(dash::LOGO))
    });
    let name = text(dash::APP_NAME, dash::TITLE_FONT, Weight::Semibold, WHITE);
    Stack::horizontal((logo, name)).style(|s| {
        s.flex_shrink(0.0_f32)
            .height(px(dash::TOPBAR_HEIGHT))
            .padding_left(20.0)
            .col_gap(12.0)
            .items_center()
    })
}

/// Nav item `i`: the icon square, the label and, on one item, the unread badge.
fn nav_item(i: usize, label: &'static str, state: State) -> impl IntoView {
    let active = move || state.nav.get() == i;
    let icon = Empty::new().style(move |s| {
        let hex = if active() {
            dash::NAV_ICON_ACTIVE
        } else {
            dash::NAV_ICON_COLOR
        };
        s.flex_shrink(0.0_f32)
            .size(px(dash::NAV_ICON), px(dash::NAV_ICON))
            .border_radius(4.0)
            .background(color(hex))
    });
    let label = Label::new(label).style(move |s| {
        let hex = if active() { WHITE } else { dash::SIDEBAR_TEXT };
        s.flex_grow(1.0_f32).color(color(hex))
    });
    let badge = if i == dash::BADGE_ITEM {
        Container::new(text(dash::BADGE_COUNT, 12.0, Weight::Semibold, WHITE))
            .style(|s| pill(s, 20.0, 7.0, dash::BADGE_BG))
            .into_any()
    } else {
        Empty::new().into_any()
    };
    Stack::horizontal((icon, label, badge))
        .style(move |s| {
            s.flex_shrink(0.0_f32)
                .width_full()
                .height(px(dash::NAV_HEIGHT))
                .padding_left(16.0)
                .padding_right(12.0)
                .col_gap(12.0)
                .items_center()
                .border_radius(px(dash::NAV_RADIUS))
                .apply_if(active(), |s| s.background(color(dash::NAV_ACTIVE)))
                .apply_if(!active(), |s| {
                    s.hover(|s| s.background(color(dash::NAV_HOVER)))
                })
        })
        .action(move || state.nav.set(i))
}

/// The user block: a rule, then the avatar with the name and role beside it.
fn user() -> impl IntoView {
    let avatar = Container::new(text(dash::USER_INITIALS, 12.0, Weight::Semibold, WHITE))
        .style(|s| circle(s, dash::USER_AVATAR, dash::USER_COLOR));
    let name = text(dash::USER_NAME, dash::FONT_SIZE, Weight::Normal, WHITE);
    let role = text(dash::USER_ROLE, 12.0, Weight::Normal, dash::SIDEBAR_MUTED);
    let lines = Stack::vertical((name, role)).style(|s| s.row_gap(2.0));
    Stack::horizontal((avatar, lines)).style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(px(dash::WINDOW.1 - dash::USER_RULE_Y))
            .padding_top(px(740.0 - dash::USER_RULE_Y - 1.0))
            .padding_left(20.0)
            .col_gap(12.0)
            .items_start()
            .border_top(1.0)
            .border_color(color(dash::SIDEBAR_RULE))
    })
}
