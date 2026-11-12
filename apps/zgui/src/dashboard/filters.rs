//! The filter card: a checkbox per status, the compare switch, and the Reset and Apply buttons.

use zgui::prelude::*;
use zgui_ui::prelude::*;

use super::Filters;
use super::spec::{self, Status};

/// The filter card. Its controls write `filters`; Reset puts every value back.
#[component]
pub fn FilterCard(filters: Filters) -> impl IntoView {
    let compare_label = NodeRef::new();
    let compare_switch = NodeRef::new();
    view! {
        column(class = "card filters") {
            text(class = "card__title") {{spec::FILTERS_TITLE}}
            column(class = "checks") {
                {Vec::from(Status::ALL.map(|status| view! {
                    StatusCheck(status = status, filters = filters)
                }.into_view()))}
            }
            row(class = "compare") {
                Label(node_ref = compare_label, control = compare_switch, class = "compare__label") {
                    {spec::COMPARE_LABEL}
                }
                Switch(
                    node_ref = compare_switch,
                    checked = filters.compare,
                    labelled_by = compare_label,
                    class = "switch"
                )
            }
            row(class = "actions") {
                Button(
                    variant = ButtonVariant::Outline,
                    class = "action reset",
                    on:click = move |_| filters.reset()
                ) {{spec::RESET_LABEL}}
                Button(class = "action apply") {{spec::APPLY_LABEL}}
            }
        }
    }
}

/// The checkbox and label of one status.
#[component]
fn StatusCheck(status: Status, filters: Filters) -> impl IntoView {
    let label = NodeRef::new();
    let check = NodeRef::new();
    view! {
        row(class = "check") {
            Checkbox(
                node_ref = check,
                checked = filters.shown[status.filter_row()],
                labelled_by = label,
                class = "check__box"
            )
            Label(node_ref = label, control = check, class = "check__label") {{status.label()}}
        }
    }
}
