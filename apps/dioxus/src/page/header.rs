//! The header: the status text on the left and the children on the right.

use dioxus::prelude::*;

/// A 40 px header that shows `status` and its children.
#[component]
pub fn Header(status: String, children: Element) -> Element {
    rsx! {
        div { class: "header",
            span { class: "status", "{status}" }
            {children}
        }
    }
}
