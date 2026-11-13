//! The orders card: the column header and one keyed row per order that passes the status filter
//! and the search, sorted by amount after a click on the amount title. Every row is a real
//! element; a row's look comes from its order alone, so a filter change or a sort keeps the rows
//! whose order stays.

use std::rc::Rc;

use dioxus::prelude::*;

use super::spec::{self, Order};
use super::{View, css_color};

/// The table of `count` orders, filtered and sorted by `view`.
#[component]
pub fn Orders(count: usize, view: View) -> Element {
    let orders: Rc<Vec<Order>> = use_hook(|| Rc::new(spec::orders(count)));
    let query = view.query.read();
    let shown = (view.shown)();
    let passing = orders
        .iter()
        .filter(|order| spec::visible(order, &shown, &query))
        .cloned()
        .collect();
    rsx! {
        div { class: "card orders",
            OrdersHead { sorted: view.sorted }
            for order in spec::arranged(passing, (view.sorted)()) {
                OrderRow { key: "{order.number}", order }
            }
        }
    }
}

/// The class of each column, in the order of `COLUMNS`.
const COLUMN_CLASSES: [&str; 5] = [
    "col-order",
    "col-customer",
    "col-status",
    "col-progress",
    "col-amount",
];

/// The column titles. A click on the amount title flips `sorted`.
#[component]
pub fn OrdersHead(sorted: Signal<bool>) -> Element {
    rsx! {
        div { class: "orders__head",
            for (index , title) in spec::COLUMNS.into_iter().enumerate() {
                if index == spec::SORT_COLUMN {
                    button {
                        key: "{index}",
                        class: "sort",
                        class: if sorted() { "sorted" },
                        onclick: move |_| sorted.toggle(),
                        {title}
                    }
                } else {
                    span { key: "{index}", class: COLUMN_CLASSES[index], {title} }
                }
            }
        }
    }
}

/// One order: its number, the customer with an avatar, the status pill, the progress bar and the
/// amount. The stylesheet gives it the hover color.
#[component]
pub fn OrderRow(order: Order) -> Element {
    let (pill_bg, pill_text) = order.status.colors();
    rsx! {
        div { class: "order",
            span { class: "col-order order__number", {order.number_text()} }
            div { class: "col-customer",
                span {
                    class: "avatar",
                    background_color: css_color(order.avatar),
                    "{order.initials}"
                }
                span { "{order.customer}" }
            }
            div { class: "col-status",
                span {
                    class: "pill status",
                    background_color: css_color(pill_bg),
                    color: css_color(pill_text),
                    {order.status.label()}
                }
            }
            div { class: "col-progress",
                div { class: "progress",
                    div { class: "progress__fill", width: "{order.progress}%" }
                }
            }
            span { class: "col-amount order__amount", {order.amount()} }
        }
    }
}
