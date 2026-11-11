//! The virtual scenario: the rows of the rows scenario in the `zgui-ui` virtualized list, which
//! builds only the rows in and near the viewport.

use zgui::prelude::*;
use zgui_ui::prelude::*;

use crate::rows::RowProps;

/// The virtualized list of `count` rows.
#[component]
pub fn VirtualRows(count: usize) -> impl IntoView {
    view! {
        VirtualList(
            count = Signal::stored_local(count),
            row_size = 32.0,
            class = "rows",
            row = move |index: usize| view! { Row(index = index) }
        )
    }
}
