//! The virtual scenario: the rows of the rows scenario in vizia's `VirtualList`, which builds only
//! the rows in view and rebinds them to new indices as the list scrolls. Its scroll view is the
//! stock one of the rows scenario, which moves 1:1 with touchpad pixel distances.
//!
//! `VirtualList::new_generic` takes the row count as its source and maps an index to itself, so
//! the list needs no `Vec` of a million items.

use vizia::prelude::*;

use crate::rows;
use crate::spec;

/// Builds the virtualized list of `count` rows.
pub fn build(cx: &mut Context, count: usize) {
    cx.add_stylesheet(include_style!("src/rows.css"))
        .expect("failed to add the rows stylesheet");
    cx.add_stylesheet(include_style!("src/virtual_rows/virtual.css"))
        .expect("failed to add the virtual stylesheet");
    VirtualList::new_generic(
        cx,
        count,
        |count: &usize| *count,
        |_: &usize, index| index,
        spec::ROW_HEIGHT,
        |cx, index, _| rows::row(cx, index),
    )
    .class("virtual");
}
