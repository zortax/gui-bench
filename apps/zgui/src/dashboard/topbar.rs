//! The top bar: the page title, the search input, the export button and the avatar.

use zgui::prelude::*;
use zgui_ui::prelude::*;

use super::{Filters, spec};

/// The white bar above the content. Typing in the search writes `filters.search`.
#[component]
pub fn TopBar(filters: Filters) -> impl IntoView {
    view! {
        row(class = "topbar") {
            text(class = "topbar__title") {{spec::PAGE_TITLE}}
            Input(
                value = filters.search,
                placeholder = spec::SEARCH_PLACEHOLDER,
                label = spec::SEARCH_PLACEHOLDER,
                class = "search"
            )
            Button(class = "export") {{spec::EXPORT_LABEL}}
            row(class = "topbar__avatar") {{spec::USER_INITIALS}}
        }
    }
}
