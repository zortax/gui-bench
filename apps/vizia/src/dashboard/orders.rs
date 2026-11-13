//! The orders card: the column header and a `List` of the orders that pass the filters, in the
//! order the table shows them. The list is bound to a memo of the filters and the sorting; when
//! the memo changes, the list rebuilds its rows from the first one that differs.

use vizia::prelude::*;

use super::{Dashboard, rgb};
use crate::dashboard_spec::{COLUMNS, Order, SORT_COLUMN, Status, arranged, orders, visible};

/// Builds the orders card with `count` orders.
pub fn build(cx: &mut Context, state: Dashboard, count: usize) {
    let orders = orders(count);
    let shown = Memo::new(move |_| {
        let statuses = state.shown.map(|shown| shown.get());
        let passing = state.search.with(|search| {
            orders
                .iter()
                .filter(|order| visible(order, &statuses, search))
                .cloned()
                .collect()
        });
        arranged(passing, state.sorted.get())
    });
    VStack::new(cx, move |cx| {
        head(cx, state.sorted);
        List::new(cx, shown, |cx, _, order| {
            row(cx, order.get());
        })
        .show_horizontal_scrollbar(false)
        .show_vertical_scrollbar(false);
    })
    .class("card")
    .class("orders");
}

/// The classes that give the columns their widths, in the order of `COLUMNS`.
const COLUMN_CLASSES: [&str; 5] = [
    "col-order",
    "col-customer",
    "col-status",
    "col-progress",
    "col-amount",
];

/// Builds the column titles. A press on the amount title flips `sorted`.
pub fn head(cx: &mut Context, sorted: Signal<bool>) {
    HStack::new(cx, move |cx| {
        for (title, class) in COLUMNS.into_iter().zip(COLUMN_CLASSES).take(SORT_COLUMN) {
            Label::new(cx, title).class("column").class(class);
        }
        HStack::new(cx, move |cx| {
            Label::new(cx, COLUMNS[SORT_COLUMN])
                .class("column")
                .class("sort")
                .toggle_class("sorted", sorted)
                .on_press(move |_| sorted.update(|sorted| *sorted = !*sorted));
        })
        .class(COLUMN_CLASSES[SORT_COLUMN]);
    })
    .class("orders-header");
}

/// Builds the row of `order`.
fn row(cx: &mut Context, order: Order) {
    HStack::new(cx, move |cx| cells(cx, order.clone())).class("order");
}

/// Builds the cells of the row of `order`. The cells are no hover targets, so the row takes the
/// `:hover` state.
pub fn cells(cx: &mut Context, order: Order) {
    let (number, amount) = (order.number_text(), order.amount());
    let Order {
        customer,
        initials,
        avatar,
        status,
        progress,
        ..
    } = order;
    Label::new(cx, number)
        .class("col-order")
        .class("order-number")
        .hoverable(false);
    HStack::new(cx, move |cx| {
        HStack::new(cx, move |cx| {
            Label::new(cx, initials);
        })
        .class("avatar")
        .background_color(rgb(avatar));
        Label::new(cx, customer);
    })
    .class("col-customer")
    .hoverable(false);
    HStack::new(cx, |cx| {
        Label::new(cx, status.label())
            .class("pill")
            .class(status_class(status));
    })
    .class("col-status")
    .hoverable(false);
    HStack::new(cx, |cx| {
        Element::new(cx)
            .class("progress-fill")
            .width(Percentage(progress as f32));
    })
    .class("col-progress")
    .class("progress")
    .hoverable(false);
    Label::new(cx, amount).class("col-amount").hoverable(false);
}

/// Returns the class that colors the pill of `status`.
fn status_class(status: Status) -> &'static str {
    match status {
        Status::Paid => "paid",
        Status::Refunded => "refunded",
        Status::Shipped => "shipped",
        Status::Pending => "pending",
    }
}
