//! The top bar: the title of the page, the built-in `Textbox` as the search input, the live
//! `Switch` with its spinner, the export menu and the avatar. The textbox shows the search signal
//! and emits `Search` on every edit, so Reset clears it through the signal.

use vizia::prelude::*;

use super::{Dashboard, DashboardEvent};
use crate::dashboard_spec::{
    EXPORT_LABEL, MENU_ITEMS, NAV_ITEMS, SEARCH_PLACEHOLDER, USER_INITIALS,
};

/// Builds the top bar.
pub fn build(cx: &mut Context, state: Dashboard) {
    HStack::new(cx, move |cx| {
        Label::new(cx, state.nav.map(|item| NAV_ITEMS[*item])).class("page-title");
        Textbox::new(cx, state.search)
            .placeholder(SEARCH_PLACEHOLDER)
            .on_edit(|cx, text| cx.emit(DashboardEvent::Search(text)))
            .class("search");
        HStack::new(cx, move |cx| {
            Switch::new(cx, state.live.on).on_toggle(|cx| cx.emit(DashboardEvent::ToggleLive));
        })
        .class("live");
        HStack::new(cx, move |cx| {
            Element::new(cx)
                .class("spinner")
                .rotate(state.live.angle)
                .display(state.live.on);
        })
        .class("spinner-cell");
        export_menu(cx);
        HStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, USER_INITIALS);
            })
            .class("topbar-avatar");
        })
        .class("topbar-avatar-cell");
    })
    .class("topbar");
}

/// Builds the export button and the menu it opens: a `Dropdown` whose trigger is the button and
/// whose popup holds one `MenuButton` per item. The button switches the popup; an item closes it.
fn export_menu(cx: &mut Context) {
    Dropdown::new(
        cx,
        |cx| {
            Button::new(cx, |cx| Label::new(cx, EXPORT_LABEL))
                .class("primary")
                .on_press(|cx| cx.emit(PopupEvent::Switch));
        },
        |cx| {
            for item in MENU_ITEMS {
                MenuButton::new(
                    cx,
                    |cx| cx.emit(PopupEvent::Close),
                    move |cx| Label::new(cx, item),
                );
            }
        },
    )
    .placement(Placement::BottomStart)
    .show_arrow(false)
    .arrow_size(Pixels(4.0))
    .class("export");
}
