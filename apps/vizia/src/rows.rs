//! The rows scenario: a full-window vertical scroll view that holds every row as a real view.
//! The stock scroll view moves the content 1:1 with touchpad pixel distances, so the scenario
//! adds no scroll handling of its own.

use vizia::prelude::*;

/// Builds the scroll view with `count` rows and adds the stylesheet.
pub fn build(cx: &mut Context, count: usize) {
    cx.add_stylesheet(include_style!("src/rows.css"))
        .expect("failed to add the rows stylesheet");
    ScrollView::new(cx, move |cx| {
        VStack::new(cx, move |cx| {
            for i in 0..count {
                row(cx, i);
            }
        })
        .class("rows");
    })
    .show_horizontal_scrollbar(false)
    .class("scroll");
}

/// Builds one row: a label and a right-aligned value. The labels are no hover targets, so the
/// row itself takes the `:hover` state. The virtual scenario builds its rows with it too.
pub fn row(cx: &mut Context, i: usize) -> Handle<'_, HStack> {
    HStack::new(cx, move |cx| {
        Label::new(cx, format!("Row {i:05}"))
            .class("label")
            .hoverable(false);
        Label::new(cx, format!("{}", (i * 7919) % 10000))
            .class("value")
            .hoverable(false);
    })
    .class("row")
    .toggle_class("odd", i % 2 == 1)
}
