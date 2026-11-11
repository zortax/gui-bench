//! One row: its label on the left and its value on the right. The stylesheet gives it the hover
//! color through `:hover`.

use dioxus::prelude::*;

use crate::spec;

/// The row at `index`, with alternating backgrounds by parity.
#[component]
pub fn Row(index: usize) -> Element {
    rsx! {
        div { class: "row", class: if index % 2 == 1 { "odd" },
            span { class: "row__label", {spec::row_label(index)} }
            span { class: "row__value", "{spec::value(index as u64)}" }
        }
    }
}
