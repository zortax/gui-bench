//! The list scenario: the header of the orders over vizia's `VirtualList`, which builds only the
//! rows in view and rebinds them to new indices as the list scrolls.
//!
//! The list is bound to a memo of the order indices in the order of the rows, so it holds no
//! `Order` values: a row makes its order when it is built, and builds its cells again when the
//! sorting gives it another index.

use std::rc::Rc;

use vizia::prelude::*;

use super::orders;
use crate::dashboard_spec::{ORDER_ROW_HEIGHT, list_order, order};

/// Builds the header and the virtual list of `count` orders and adds the stylesheet.
pub fn build(cx: &mut Context, count: usize) {
    super::add_stylesheet(cx);
    let sorted = Signal::new(false);
    let indices = sorted.map(move |sorted| Rc::<[u32]>::from(list_order(count, *sorted)));
    VStack::new(cx, move |cx| {
        orders::head(cx, sorted);
        VirtualList::new(cx, indices, ORDER_ROW_HEIGHT, |cx, _, index| {
            HStack::new(cx, move |cx| {
                Binding::new(cx, index, move |cx| {
                    orders::cells(cx, order(index.get() as usize));
                });
            })
            .class("order")
        });
    })
    .class("list");
}
