//! The sidebar: its own view, since only a nav click or a hover changes it. The root embeds it
//! as a cached view, so neither re-renders the rest of the dashboard.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::spec;

/// The sidebar and its active nav item.
#[derive(Default)]
pub struct Sidebar {
    active: usize,
}

impl Sidebar {
    /// Makes nav item `i` the active one.
    fn select(&mut self, i: usize, cx: &mut Context<Self>) {
        if self.active != i {
            self.active = i;
            cx.notify();
        }
    }

    /// Returns nav item `i`: a hover color unless it is active, and a click that activates it.
    fn item(&self, i: usize, cx: &mut Context<Self>) -> Stateful<Div> {
        let active = i == self.active;
        let (background, icon, text) = if active {
            (spec::NAV_ACTIVE, spec::NAV_ICON_ACTIVE, 0xffffff)
        } else {
            (spec::SIDEBAR_BG, spec::NAV_ICON_COLOR, spec::SIDEBAR_TEXT)
        };
        h_flex()
            .id(("nav", i))
            .flex_none()
            .w(px(spec::NAV_WIDTH))
            .h(px(spec::NAV_HEIGHT))
            .pl(px(16.0))
            .pr(px(12.0))
            .gap(px(12.0))
            .rounded(px(spec::NAV_RADIUS))
            .bg(rgb(background))
            .when(!active, |item| {
                item.hover(|style| style.bg(rgb(spec::NAV_HOVER)))
            })
            .on_click(cx.listener(move |this, _, _, cx| this.select(i, cx)))
            .child(
                div()
                    .flex_none()
                    .size(px(spec::NAV_ICON))
                    .rounded(px(4.0))
                    .bg(rgb(icon)),
            )
            .child(
                div()
                    .flex_1()
                    .text_color(rgb(text))
                    .child(spec::NAV_ITEMS[i]),
            )
            .when(i == spec::BADGE_ITEM, |item| item.child(badge()))
    }
}

/// Returns the unread badge.
fn badge() -> Div {
    h_flex()
        .flex_none()
        .h(px(20.0))
        .px(px(7.0))
        .rounded(px(10.0))
        .bg(rgb(spec::BADGE_BG))
        .text_size(px(12.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(0xffffff))
        .child(spec::BADGE_COUNT.to_string())
}

/// Returns the brand at the top: the logo square and the app name.
fn brand() -> Div {
    h_flex()
        .flex_none()
        .h(px(64.0))
        .pl(px(20.0))
        .gap(px(12.0))
        .child(
            div()
                .flex_none()
                .size(px(spec::LOGO_SIZE))
                .rounded(px(spec::LOGO_RADIUS))
                .bg(rgb(spec::LOGO)),
        )
        .child(
            div()
                .text_size(px(spec::TITLE_FONT))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xffffff))
                .child(spec::APP_NAME),
        )
}

/// Returns the user block at the bottom: the rule, the avatar, the name and the role.
fn user() -> Div {
    h_flex()
        .flex_none()
        .h(px(spec::WINDOW.1 - spec::USER_RULE_Y))
        .items_start()
        .pt(px(19.0))
        .pl(px(20.0))
        .gap(px(12.0))
        .border_t_1()
        .border_color(rgb(spec::SIDEBAR_RULE))
        .child(
            h_flex()
                .flex_none()
                .size(px(spec::USER_AVATAR))
                .justify_center()
                .rounded_full()
                .bg(rgb(spec::USER_COLOR))
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xffffff))
                .child(spec::USER_INITIALS),
        )
        .child(
            v_flex()
                .h(px(spec::USER_AVATAR))
                .justify_center()
                .child(div().text_color(rgb(0xffffff)).child(spec::USER_NAME))
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(rgb(spec::SIDEBAR_MUTED))
                        .child(spec::USER_ROLE),
                ),
        )
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items: Vec<_> = (0..spec::NAV_ITEMS.len())
            .map(|i| self.item(i, cx))
            .collect();
        v_flex()
            .size_full()
            .bg(rgb(spec::SIDEBAR_BG))
            .text_size(px(spec::FONT_SIZE))
            .child(brand())
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .pt(px(spec::NAV_TOP - 64.0))
                    .px(px(spec::NAV_X))
                    .gap(px(spec::NAV_STRIDE - spec::NAV_HEIGHT))
                    .overflow_hidden()
                    .children(items),
            )
            .child(user())
    }
}
