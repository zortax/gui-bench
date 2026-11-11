//! One row: its label on the left and its value on the right.

use zgui::prelude::*;

/// The multiplier that spreads the row values over 0 to 9999.
const VALUE_STEP: usize = 7919;

/// The row at `index`, with alternating backgrounds by parity.
#[component]
pub fn Row(index: usize) -> impl IntoView {
    view! {
        row(class = "row", class:odd = index % 2 == 1) {
            text(class = "row__label") {{format!("Row {index:05}")}}
            text(class = "row__value") {{(index * VALUE_STEP % 10000).to_string()}}
        }
    }
}
