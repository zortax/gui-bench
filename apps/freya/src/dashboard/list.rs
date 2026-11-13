//! The list scenario: the header of the orders over freya's `VirtualScrollView`, which builds
//! only the order rows in the viewport.

use std::rc::Rc;

use freya::prelude::*;

use super::orders::{OrderRow, OrdersHead};
use super::{color, root, spec};

/// The root of the list with `count` rows. A click on the amount title sorts the data; the
/// order of the data is the data of the virtual scroll view, so a sort rebuilds the visible rows.
pub struct OrderList {
    pub count: usize,
}

impl App for OrderList {
    fn render(&self) -> impl IntoElement {
        let count = self.count;
        let sorted = use_state(|| false);
        let order = use_memo(move || Rc::new(spec::list_order(count, sorted())));

        root()
            .background(color(spec::CARD_BG))
            .child(OrdersHead {
                sorted,
                rounded: false,
            })
            .child(
                VirtualScrollView::new_with_data(
                    order.read().clone(),
                    |item: VirtualItem, order: &Rc<Vec<u32>>| {
                        OrderRow::new(spec::order(order[item.index] as usize))
                            .key(item.index)
                            .into()
                    },
                )
                .length(count)
                .item_size(spec::ORDER_ROW_HEIGHT),
            )
    }
}
