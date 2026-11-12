//! The top bar: the page title, the built-in `Textbox` as the search input, the export button
//! and the avatar. The textbox shows the search signal and emits `Search` on every edit, so Reset
//! clears it through the signal.

use vizia::prelude::*;

use super::{Dashboard, DashboardEvent};
use crate::dashboard_spec::{EXPORT_LABEL, PAGE_TITLE, SEARCH_PLACEHOLDER, USER_INITIALS};

/// Builds the top bar.
pub fn build(cx: &mut Context, state: Dashboard) {
    HStack::new(cx, move |cx| {
        Label::new(cx, PAGE_TITLE).class("page-title");
        HStack::new(cx, move |cx| {
            Textbox::new(cx, state.search)
                .placeholder(SEARCH_PLACEHOLDER)
                .on_edit(|cx, text| cx.emit(DashboardEvent::Search(text)))
                .class("search");
            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, EXPORT_LABEL))
                    .class("primary")
                    .class("export");
                HStack::new(cx, |cx| {
                    Label::new(cx, USER_INITIALS);
                })
                .class("topbar-avatar");
            })
            .class("topbar-actions");
        })
        .class("topbar-tools");
    })
    .class("topbar");
}
