//! The virtual scenario: the rows of the rows scenario, of which only the visible ones exist.
//! Dioxus has no virtualized list, so this is the manual approach: the scroll event gives the
//! offset, the component renders the rows in the viewport and a few around it, keyed by index,
//! and two spacers stand in for the rows above and below.

use dioxus::prelude::*;

use crate::rows::{CSS, Row};
use crate::spec;

/// The rows built above and below the viewport, so a scroll that the next render has not seen yet
/// still shows rows.
const OVERSCAN: usize = 8;

/// The scroll offset and the height of the scroll container.
#[derive(Clone, Copy, PartialEq)]
struct Viewport {
    top: f64,
    height: f64,
}

/// The scroll container of `count` rows.
#[component]
pub fn VirtualRows(count: usize) -> Element {
    let mut viewport = use_signal(|| Viewport {
        top: 0.0,
        height: f64::from(spec::WINDOW.1),
    });

    let Viewport { top, height } = viewport();
    let row = f64::from(spec::ROW_HEIGHT);
    let first = ((top / row) as usize).saturating_sub(OVERSCAN).min(count);
    let end = ((((top + height) / row).ceil() as usize) + OVERSCAN).min(count);
    let above = first as f64 * row;
    let below = (count - end) as f64 * row;
    rsx! {
        style { {CSS} }
        div {
            class: "rows",
            onscroll: move |event| {
                viewport.set(Viewport {
                    top: event.scroll_top(),
                    height: f64::from(event.client_height()),
                })
            },
            div { style: "height: {above}px" }
            for index in first..end {
                Row { key: "{index}", index }
            }
            div { style: "height: {below}px" }
        }
    }
}
