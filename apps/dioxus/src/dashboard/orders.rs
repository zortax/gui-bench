//! The orders card: the column header and one keyed row per order that passes the status filter
//! and the search. Every row is a real element; a row's look comes from its order alone, so a
//! filter change keeps the rows whose order stays.

use std::rc::Rc;

use dioxus::prelude::*;

use super::css_color;
use super::spec::{self, Order};

/// The table of `count` orders, filtered by `query` and `shown`.
#[component]
pub fn Orders(count: usize, query: ReadSignal<String>, shown: ReadSignal<[bool; 4]>) -> Element {
    let orders: Rc<Vec<Order>> = use_hook(|| Rc::new(spec::orders(count)));
    let query = query.read();
    let shown = shown();
    rsx! {
        div { class: "card orders",
            div { class: "orders__head",
                for (index , title) in spec::COLUMNS.into_iter().enumerate() {
                    span { key: "{index}", class: COLUMN_CLASSES[index], {title} }
                }
            }
            for order in orders.iter().filter(|order| spec::visible(order, &shown, &query)) {
                OrderRow { key: "{order.number}", order: order.clone() }
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

/// One order: its number, the customer with an avatar, the status pill, the progress bar and the
/// amount. The stylesheet gives it the hover color.
#[component]
fn OrderRow(order: Order) -> Element {
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
