//! The scroll container that fills the window and holds every row.

use dioxus::prelude::*;

use super::wheel::use_detent_scroll;
use super::{CSS, Row};

/// A vertical scroll container with `count` rows, each built as a real element.
#[component]
pub fn Rows(count: usize) -> Element {
    let wheel = use_detent_scroll();
    rsx! {
        style { {CSS} }
        div {
            class: "rows",
            onmounted: move |event| wheel.mounted(event),
            onwheel: move |event| wheel.scroll(event, |_| {}),
            for index in 0..count {
                Row { key: "{index}", index }
            }
        }
    }
}
