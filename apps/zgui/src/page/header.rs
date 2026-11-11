//! The header: the status text on the left and the children on the right.

use zgui::prelude::*;

/// A 40 px header that shows `status` and its children.
#[component]
pub fn Header(#[prop(into)] status: Signal<String>, children: Children) -> impl IntoView {
    view! {
        row(class = "header") {
            text(class = "status") {{move || status.get()}}
            {children.into_view_once()}
        }
    }
}
