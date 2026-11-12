//! The sidebar: the brand, the nav items and the user at the bottom. A nav item is a row that
//! takes the `active` class from the nav signal and emits `Navigate` when pressed; its hover
//! color is a `:hover` rule without a transition.

use vizia::prelude::*;

use super::{Dashboard, DashboardEvent};
use crate::dashboard_spec::{
    APP_NAME, BADGE_COUNT, BADGE_ITEM, NAV_ITEMS, USER_INITIALS, USER_NAME, USER_ROLE,
};

/// Builds the sidebar.
pub fn build(cx: &mut Context, state: Dashboard) {
    VStack::new(cx, move |cx| {
        HStack::new(cx, |cx| {
            Element::new(cx).class("logo");
            Label::new(cx, APP_NAME).class("app-name");
        })
        .class("brand");

        VStack::new(cx, move |cx| {
            for (i, name) in NAV_ITEMS.into_iter().enumerate() {
                nav_item(cx, state, i, name);
            }
        })
        .class("nav");

        HStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, USER_INITIALS);
            })
            .class("user-avatar");
            VStack::new(cx, |cx| {
                Label::new(cx, USER_NAME).class("user-name");
                Label::new(cx, USER_ROLE).class("user-role");
            })
            .class("user-text");
        })
        .class("user");
    })
    .class("sidebar");
}

/// Builds nav item `i`: the icon, the label and, on the badge item, the unread count.
fn nav_item(cx: &mut Context, state: Dashboard, i: usize, name: &'static str) {
    HStack::new(cx, move |cx| {
        Element::new(cx).class("nav-icon").hoverable(false);
        Label::new(cx, name).class("nav-label").hoverable(false);
        if i == BADGE_ITEM {
            Label::new(cx, BADGE_COUNT)
                .class("nav-badge")
                .hoverable(false);
        }
    })
    .class("nav-item")
    .toggle_class("active", state.nav.map(move |active| *active == i))
    .on_press(move |cx| cx.emit(DashboardEvent::Navigate(i)));
}
