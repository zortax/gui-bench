//! The list scenario: the header of the orders over GPUI's `uniform_list`, which builds only the
//! order rows in the window and scrolls as it does by default.

use std::ops::Range;

use gpui_kit::component::v_flex;
use gpui_kit::*;

use super::orders::{Row, header, row};
use super::spec;

/// The orders of a list of `count` rows.
pub struct OrderList {
    /// The index of the order of each row.
    order: Vec<u32>,
    /// Whether the rows are sorted by amount.
    sorted: bool,
}

impl OrderList {
    pub fn new(count: usize) -> Self {
        Self {
            order: spec::list_order(count, false),
            sorted: false,
        }
    }

    /// Sorts the rows by amount, or brings the order of the data back.
    fn toggle_sorted(&mut self, cx: &mut Context<Self>) {
        self.sorted = !self.sorted;
        self.order = spec::list_order(self.order.len(), self.sorted);
        cx.notify();
    }

    /// Returns the rows of `range`, each with its order made now.
    fn rows(&self, range: Range<usize>) -> Vec<Stateful<Div>> {
        range
            .map(|at| row(at, &Row::new(spec::order(self.order[at] as usize))))
            .collect()
    }
}

impl Render for OrderList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = uniform_list(
            "orders",
            self.order.len(),
            cx.processor(|this, range: Range<usize>, _, _| this.rows(range)),
        )
        .flex_1()
        .min_h_0()
        .w_full();
        v_flex()
            .size_full()
            .bg(rgb(spec::CARD_BG))
            .text_size(px(spec::FONT_SIZE))
            .text_color(rgb(spec::TEXT))
            .child(header(
                self.sorted,
                cx.listener(|this, _, _, cx| this.toggle_sorted(cx)),
            ))
            .child(rows)
    }
}
