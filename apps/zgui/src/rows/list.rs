//! The scroll container that fills the window and holds every row.

use zgui::prelude::*;

use super::row::RowProps;

/// A vertical scroll container with `count` rows, each built as a real element.
#[component]
pub fn Rows(count: usize) -> impl IntoView {
    view! {
        scroll(class = "rows") {
            for index in move || 0..count, key = |index: &usize| *index {
                Row(index = index)
            }
        }
    }
}
