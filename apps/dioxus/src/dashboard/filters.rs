//! The filter card: a built-in checkbox per status, the compare switch, and the Reset and Apply
//! buttons.

use dioxus::prelude::*;

use super::View;
use super::spec::{self, Status};
use super::switch::Switch;

/// The filters of the orders and the chart.
#[component]
pub fn Filters(view: View) -> Element {
    rsx! {
        div { class: "card filters",
            span { class: "card__title", {spec::FILTERS_TITLE} }
            div { class: "checks",
                for (row , status) in Status::ALL.into_iter().enumerate() {
                    StatusCheck { key: "{row}", row, status, shown: view.shown }
                }
            }
            div { class: "compare",
                span { {spec::COMPARE_LABEL} }
                Switch { on: view.compare, label: spec::COMPARE_LABEL }
            }
            div { class: "actions",
                button {
                    class: "button button--secondary",
                    onclick: move |_| view.reset(),
                    {spec::RESET_LABEL}
                }
                button { class: "button button--primary", {spec::APPLY_LABEL} }
            }
        }
    }
}

/// The checkbox of `status`, which is row `row` of `shown`.
#[component]
fn StatusCheck(row: usize, status: Status, shown: Signal<[bool; 4]>) -> Element {
    let checked = shown.read()[row];
    rsx! {
        label { class: "check",
            input {
                class: "check__box",
                r#type: "checkbox",
                checked,
                oninput: move |event| shown.with_mut(|shown| shown[row] = event.checked()),
            }
            span { {status.label()} }
        }
    }
}
