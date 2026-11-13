//! The switch. HTML has no switch element, so it is a button with the `switch` role and a knob,
//! as on the web.

use dioxus::prelude::*;

/// A switch bound to `on`; a click toggles it. `label` is its accessible name.
#[component]
pub fn Switch(on: Signal<bool>, label: &'static str) -> Element {
    let checked = on();
    rsx! {
        button {
            class: "switch",
            class: if checked { "on" },
            role: "switch",
            aria_checked: checked,
            aria_label: label,
            onclick: move |_| on.toggle(),
            span { class: "switch__knob" }
        }
    }
}
