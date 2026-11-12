//! The scroll container that fills the window and holds every row.

use dioxus::prelude::*;

use super::{CSS, Row};

/// A vertical scroll container with `count` rows, each built as a real element.
#[component]
pub fn Rows(count: usize) -> Element {
    rsx! {
        style { {CSS} }
        div { class: "rows",
            for index in 0..count {
                Row { key: "{index}", index }
            }
        }
    }
}
