//! A static row of the header scenarios: its label on the left and its value on the right.

use dioxus::prelude::*;

use crate::spec::Row;

/// A row that never changes. Its props are the row, so a parent render keeps it as it is.
#[component]
pub fn PlainRow(row: Row) -> Element {
    rsx! {
        div { class: "row",
            span { class: "row__label", "{row.label}" }
            span { class: "row__value", "{row.value}" }
        }
    }
}
