//! The page of the header scenarios: a header with the status text on the left and a slot on the
//! right, above a scroll view of rows. Rows here have one background and no hover color.

use vizia::prelude::*;

use crate::spec::Row;

/// Adds the stylesheet of the header, the rows, the spinner and the search field.
pub fn add_sheet(cx: &mut Context) {
    cx.add_stylesheet(include_style!("src/page/page.css"))
        .expect("failed to add the page stylesheet");
}

/// Builds the page: the header with `status` and the `right` slot, and the `body` below it,
/// which fills the rest of the window.
pub fn build<T: ToStringLocalized + 'static>(
    cx: &mut Context,
    status: impl Res<T> + Clone + 'static,
    right: impl FnOnce(&mut Context),
    body: impl FnOnce(&mut Context),
) {
    add_sheet(cx);
    VStack::new(cx, move |cx| {
        HStack::new(cx, move |cx| {
            Label::new(cx, status).class("status");
            right(cx);
        })
        .class("header");
        body(cx);
    })
    .class("page");
}

/// Builds a scroll view that fills the rest of the page, around a stack of rows.
pub fn scroll(cx: &mut Context, rows: impl 'static + FnOnce(&mut Context)) {
    ScrollView::new(cx, move |cx| {
        VStack::new(cx, rows).class("rows");
    })
    .show_horizontal_scrollbar(false)
    .class("list");
}

/// Builds a row that never changes: its label on the left and its value on the right.
pub fn plain_row<'a>(cx: &'a mut Context, row: &Row) -> Handle<'a, HStack> {
    let (label, value) = (row.label.clone(), row.value);
    HStack::new(cx, move |cx| {
        Label::new(cx, label).class("label");
        Label::new(cx, value).class("value");
    })
    .class("row")
}
