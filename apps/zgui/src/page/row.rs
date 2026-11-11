//! A static row of the header scenarios: its label on the left and its value on the right.

use zgui::prelude::*;

use crate::spec::Row;

/// A row that never changes.
#[component]
pub fn PlainRow(row: Row) -> impl IntoView {
    view! {
        row(class = "row") {
            text(class = "row__label") {{row.label}}
            text(class = "row__value") {{row.value.to_string()}}
        }
    }
}
