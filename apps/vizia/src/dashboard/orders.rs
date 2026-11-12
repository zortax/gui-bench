//! The orders card: the column header and one row per order, every row a real view. A row
//! shows while a memo of the filters says so (`display`), so filtering hides and shows rows
//! without rebuilding them. The cells are no hover targets, so the row takes the `:hover` state,
//! as in the rows scenario.

use vizia::prelude::*;

use super::{Dashboard, rgb};
use crate::dashboard_spec::{COLUMNS, Order, Status, orders};

/// Builds the orders card with `count` orders.
pub fn build(cx: &mut Context, state: Dashboard, count: usize) {
    VStack::new(cx, move |cx| {
        HStack::new(cx, |cx| {
            for (title, class) in COLUMNS.into_iter().zip(COLUMN_CLASSES) {
                Label::new(cx, title).class("column").class(class);
            }
        })
        .class("orders-header");
        for order in orders(count) {
            row(cx, state, order);
        }
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

/// Builds the row of `order`.
fn row(cx: &mut Context, state: Dashboard, order: Order) {
    let visible = state.visible(order.clone());
    let (number, amount) = (order.number_text(), order.amount());
    let Order {
        customer,
        initials,
        avatar,
        status,
        progress,
        ..
    } = order;
    HStack::new(cx, move |cx| {
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
    })
    .class("order")
    .display(visible);
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
