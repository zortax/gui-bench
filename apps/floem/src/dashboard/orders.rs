//! The orders card: the column header and a keyed `dyn_stack` of the orders that pass the status
//! filter and the search. Every shown order is a real row view; a filter change keeps the views
//! of the orders that stay. A row takes the hover color at once while the pointer is over it.

use std::rc::Rc;

use floem::prelude::*;
use floem::taffy::Overflow;

use super::State;
use super::style::{self, Weight, circle, color, pill, px, text};
use crate::dash::{self, Order};

/// The widths of the order, customer, status and progress columns; the amount takes the rest.
const ORDER_WIDTH: f32 = dash::COL_CUSTOMER - dash::COL_ORDER;
const CUSTOMER_WIDTH: f32 = dash::COL_STATUS - dash::COL_CUSTOMER;
const STATUS_WIDTH: f32 = dash::COL_PROGRESS - dash::COL_STATUS;
const PROGRESS_COLUMN: f32 = 200.0;
/// The padding on the right of the amount column, from a 992 px card.
const PADDING_RIGHT: f32 = 992.0 - dash::COL_AMOUNT_END;

/// Returns the orders card with `count` orders.
pub fn view(count: usize, state: State) -> impl IntoView {
    let orders = Rc::new(dash::orders(count));
    let rows = dyn_stack(
        move || {
            let shown = state.shown();
            state.search.with(|search| {
                orders
                    .iter()
                    .filter(|order| dash::visible(order, &shown, search))
                    .cloned()
                    .collect::<Vec<Order>>()
            })
        },
        |order| order.number,
        row,
    )
    .style(|s| s.flex_col().width_full());
    // The card clips the rows to its rounded corners.
    Stack::vertical((header(), rows)).style(|s| {
        style::card(s)
            .width_full()
            .overflow_x(Overflow::Clip)
            .overflow_y(Overflow::Clip)
    })
}

/// The header with the column titles.
fn header() -> impl IntoView {
    let [order, customer, status, progress, amount] =
        dash::COLUMNS.map(|title| text(title, 12.0, Weight::Semibold, dash::MUTED));
    cells(
        (order, customer, status, progress, amount),
        dash::TABLE_HEADER_HEIGHT,
    )
    .style(|s| {
        s.border_bottom(1.0)
            .border_color(color(dash::RULE))
            .background(color(dash::TABLE_HEADER_BG))
            .border_top_left_radius(px(dash::CARD_RADIUS))
            .border_top_right_radius(px(dash::CARD_RADIUS))
    })
}

/// One order row.
fn row(order: Order) -> impl IntoView {
    let number = text(order.number_text(), dash::FONT_SIZE, Weight::Normal, dash::MUTED);
    let avatar = Container::new(text(&order.initials, 11.0, Weight::Semibold, 0xffffff))
        .style(move |s| circle(s, dash::AVATAR, order.avatar));
    let name = text(&order.customer, dash::FONT_SIZE, Weight::Normal, dash::TEXT);
    let customer = Stack::horizontal((avatar, name))
        .style(|s| s.items_center().col_gap(px(40.0 - dash::AVATAR)));
    let (pill_bg, pill_text) = order.status.colors();
    let status = Container::new(text(order.status.label(), 12.0, Weight::Semibold, pill_text))
        .style(move |s| pill(s, 22.0, 10.0, pill_bg));
    let fill = Empty::new().style(move |s| {
        s.height_full()
            .width((f64::from(order.progress)).pct())
            .border_radius(px(dash::PROGRESS_HEIGHT / 2.0))
            .background(color(dash::PRIMARY))
    });
    let progress = Container::new(fill).style(|s| {
        s.size(px(dash::PROGRESS_WIDTH), px(dash::PROGRESS_HEIGHT))
            .border_radius(px(dash::PROGRESS_HEIGHT / 2.0))
            .background(color(dash::TRACK))
    });
    let amount = text(order.amount(), dash::FONT_SIZE, Weight::Semibold, dash::TEXT);
    cells(
        (number, customer, status, progress, amount),
        dash::ORDER_ROW_HEIGHT,
    )
    .style(|s| {
        s.border_bottom(1.0)
            .border_color(color(dash::ROW_RULE))
            .background(color(dash::CARD_BG))
            .hover(|s| s.background(color(dash::ROW_HOVER)))
    })
}

/// Lays out the five cells of a header or row in their columns, `height` tall.
fn cells(
    (order, customer, status, progress, amount): (
        impl IntoView + 'static,
        impl IntoView + 'static,
        impl IntoView + 'static,
        impl IntoView + 'static,
        impl IntoView + 'static,
    ),
    height: f32,
) -> Stack {
    let column = |view: Box<dyn View>, width: f32| {
        Container::new(view).style(move |s| s.flex_shrink(0.0_f32).width(px(width)).items_center())
    };
    let amount = Container::new(amount).style(|s| {
        s.flex_grow(1.0_f32)
            .min_width(0.0)
            .justify_end()
            .items_center()
    });
    Stack::horizontal((
        column(order.into_any(), ORDER_WIDTH),
        column(customer.into_any(), CUSTOMER_WIDTH),
        column(status.into_any(), STATUS_WIDTH),
        column(progress.into_any(), PROGRESS_COLUMN),
        amount,
    ))
    .style(move |s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(px(height))
            .padding_left(px(dash::COL_ORDER))
            .padding_right(px(PADDING_RIGHT))
            .items_center()
    })
}
