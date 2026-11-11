//! The scroll container that fills the window and holds every row.

use zgui::prelude::*;

use super::row::RowProps;
use super::wheel;

/// A vertical scroll container with `count` rows, each built as a real element.
#[component]
pub fn Rows(count: usize) -> impl IntoView {
    let list = NodeRef::new();
    view! {
        scroll(
            class = "rows",
            node_ref = list,
            on:wheel = move |ev| {
                if wheel::scroll_detents(ev, &list) {
                    ev.prevent_default();
                }
            }
        ) {
            for index in move || 0..count, key = |index: &usize| *index {
                Row(index = index)
            }
        }
    }
}
