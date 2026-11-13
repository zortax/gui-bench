//! The list scenario: the header of the orders over floem's `VirtualStack`, which builds only the
//! order rows in and near the window. The stack takes the indices of the orders in the order of
//! its rows and a fixed row height, and scrolls in floem's scroll container.

use floem::imbl::Vector;
use floem::prelude::*;
use floem::reactive::Memo;

use super::orders::{header, row};
use super::style::{self, color, px};
use crate::dash;

/// Returns the window content: the header and the virtualized list of `count` order rows. A click
/// on the amount title sorts the indices; a row makes its order when it is built.
pub fn view(count: usize) -> impl IntoView {
    let sorted = RwSignal::new(false);
    let order = Memo::new(move |_| Vector::from(dash::list_order(count, sorted.get())));
    let rows = VirtualStack::with_view(
        move || order.get(),
        |index: u32| row(dash::order(index as usize)),
    )
    .item_size_fixed(|| px(dash::ORDER_ROW_HEIGHT))
    .style(|s| s.flex_col().width_full())
    .scroll()
    .style(|s| s.flex_grow(1.0_f32).min_height(0.0).width_full());
    Stack::vertical((header(sorted), rows))
        .style(|s| style::window(s).background(color(dash::CARD_BG)))
}
