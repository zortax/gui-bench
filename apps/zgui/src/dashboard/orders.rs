//! The orders card: the column titles and one row per order that passes the filters.

use std::rc::Rc;

use zgui::prelude::*;

use super::spec::{self, Order, Status};
use super::{Filters, hex};

/// The class of each column, in the order of `COLUMNS`.
const COLUMN_CLASSES: [&str; 5] = [
    "col col-order",
    "col col-customer",
    "col col-status",
    "col col-progress",
    "col col-amount",
];

/// The orders card with `count` orders. A keyed `for` shows the ones that pass the status
/// checkboxes and the search, so a filter change or a sort moves rows rather than rebuilding
/// them. A click on the amount title sorts by amount.
#[component]
pub fn OrdersTable(count: usize, filters: Filters) -> impl IntoView {
    let orders = Rc::new(spec::orders(count));
    view! {
        column(class = "card orders") {
            OrdersHead(sorted = filters.sorted)
            for order in move || shown(&orders, filters), key = |order: &Order| order.number {
                OrderRow(order = order)
            }
        }
    }
}

/// The column titles. A click on the amount title flips `sorted`.
#[component]
pub fn OrdersHead(sorted: RwSignal<bool>) -> impl IntoView {
    view! {
        row(class = "orders__head") {
            {Vec::from(std::array::from_fn::<_, 4, _>(|k| view! {
                row(class = COLUMN_CLASSES[k]) {{spec::COLUMNS[k]}}
            }.into_view()))}
            row(class = COLUMN_CLASSES[spec::SORT_COLUMN]) {
                control(
                    class = "sort",
                    class:sorted = move || sorted.get(),
                    on:click = move |_| sorted.update(|sorted| *sorted = !*sorted)
                ) {{spec::COLUMNS[spec::SORT_COLUMN]}}
            }
        }
    }
}

/// Returns the orders that pass the filters, in the order the table shows them, tracking the
/// filters and the sorting.
fn shown(orders: &[Order], filters: Filters) -> Vec<Order> {
    let statuses = filters.statuses();
    let passing = filters.search.with(|search| {
        orders
            .iter()
            .filter(|order| spec::visible(order, &statuses, search))
            .cloned()
            .collect()
    });
    spec::arranged(passing, filters.sorted.get())
}

/// One order: number, customer with avatar, status pill, progress bar and amount.
#[component]
pub fn OrderRow(order: Order) -> impl IntoView {
    view! {
        row(class = "order") {
            row(class = "col col-order") {{order.number_text()}}
            row(class = "col col-customer") {
                row(class = "avatar", style:background-color = hex(order.avatar)) {{order.initials.clone()}}
                text(class = "customer") {{order.customer.clone()}}
            }
            row(class = "col col-status") {
                row(class = pill_class(order.status)) {{order.status.label()}}
            }
            row(class = "col col-progress") {
                box(class = "progress") {
                    box(class = "progress__fill", style:width = format!("{}%", order.progress))
                }
            }
            row(class = "col col-amount") {{order.amount()}}
        }
    }
}

/// Returns the classes of the status pill, whose colors the sheet gives per status.
fn pill_class(status: Status) -> &'static str {
    match status {
        Status::Paid => "pill paid",
        Status::Refunded => "pill refunded",
        Status::Shipped => "pill shipped",
        Status::Pending => "pill pending",
    }
}
