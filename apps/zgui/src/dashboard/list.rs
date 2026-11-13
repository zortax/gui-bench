//! The list scenario: the header of the orders over the `zgui-ui` virtual list, which builds only
//! the order rows in and near the window.

use std::sync::Arc;

use zgui::prelude::*;
use zgui_ui::prelude::*;
use zgui_ui_tokens::prelude::*;

use super::orders::{OrderRowProps, OrdersHeadProps};
use super::{TOKENS, spec};

/// The orders of a list of `count` rows. A click on the amount title sorts the data; every row
/// on screen reads the order of the data, so a sort changes those rows and no others.
#[component]
pub fn OrderList(count: usize) -> impl IntoView {
    let sorted = RwSignal::new(false);
    let order = Memo::new(move |_| Arc::new(spec::list_order(count, sorted.get())));
    view! {
        ThemeProvider(light = Theme::light().with_css(TOKENS), scheme = ColorScheme::Light) {
            column(class = "list") {
                OrdersHead(sorted = sorted)
                VirtualList(
                    count = Signal::stored_local(count),
                    row_size = spec::ORDER_ROW_HEIGHT,
                    label = "Orders",
                    class = "list__rows",
                    row = move |index: usize| {
                        move || {
                            let data = order.with(|order| order[index] as usize);
                            view! { OrderRow(order = spec::order(data)) }
                        }
                    }
                )
            }
        }
    }
}
