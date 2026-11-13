//! The list scenario: the header of the orders over a scroll container with `count` order rows,
//! of which only the visible ones exist. Dioxus has no virtualized list, so this is the manual
//! approach: the scroll event gives the offset, the component renders the rows in the viewport
//! and a few around it, keyed by order, and two spacers stand in for the rows above and below.

use std::rc::Rc;

use dioxus::prelude::*;

use super::CSS;
use super::orders::{OrderRow, OrdersHead};
use super::spec;

/// The rows built above and below the viewport, so a scroll that the next render has not seen yet
/// still shows rows.
const OVERSCAN: usize = 8;

/// The scroll offset and the height of the scroll container.
#[derive(Clone, Copy, PartialEq)]
struct Viewport {
    top: f64,
    height: f64,
}

/// The orders of a list of `count` rows. A click on the amount title sorts the data; each row is
/// made from its index when it comes into view.
#[component]
pub fn OrderList(count: usize) -> Element {
    let sorted = use_signal(|| false);
    let order = use_memo(move || Rc::new(spec::list_order(count, sorted())));
    let mut viewport = use_signal(|| Viewport {
        top: 0.0,
        height: f64::from(spec::WINDOW.1 - spec::LIST_ROWS_TOP),
    });

    let Viewport { top, height } = viewport();
    let row = f64::from(spec::ORDER_ROW_HEIGHT);
    let first = ((top / row) as usize).saturating_sub(OVERSCAN).min(count);
    let end = ((((top + height) / row).ceil() as usize) + OVERSCAN).min(count);
    let above = first as f64 * row;
    let below = (count - end) as f64 * row;
    let order = order();
    rsx! {
        style { {CSS} }
        div { class: "list",
            OrdersHead { sorted }
            div {
                class: "list__rows",
                onscroll: move |event| {
                    viewport.set(Viewport {
                        top: event.scroll_top(),
                        height: f64::from(event.client_height()),
                    })
                },
                div { style: "height: {above}px" }
                for index in order[first..end].iter().map(|index| *index as usize) {
                    OrderRow { key: "{index}", order: spec::order(index) }
                }
                div { style: "height: {below}px" }
            }
        }
    }
}
