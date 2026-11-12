//! The orders card: its own view that holds every order and the filter, and shows the header and
//! one real row per visible order, each with an avatar, a status pill and a progress bar.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::spec::{self, Order};
use super::style::card;

/// The column widths, from the left edge of each column to the next one.
const ORDER_WIDTH: f32 = spec::COL_CUSTOMER - spec::COL_ORDER;
const CUSTOMER_WIDTH: f32 = spec::COL_STATUS - spec::COL_CUSTOMER;
const STATUS_WIDTH: f32 = spec::COL_PROGRESS - spec::COL_STATUS;
/// The customer name from the left of the avatar.
const NAME_OFFSET: f32 = 40.0;
/// The space right of the amount column in a full-width card.
const AMOUNT_PADDING: f32 =
    spec::WINDOW.0 - spec::SIDEBAR_WIDTH - 2.0 * spec::CONTENT_PADDING - spec::COL_AMOUNT_END;

/// One order with its texts, made once.
struct Row {
    order: Order,
    number: SharedString,
    customer: SharedString,
    initials: SharedString,
    amount: SharedString,
}

impl Row {
    fn new(order: Order) -> Self {
        Self {
            number: order.number_text().into(),
            customer: order.customer.clone().into(),
            initials: order.initials.clone().into(),
            amount: order.amount().into(),
            order,
        }
    }
}

/// The orders and the filter on them.
pub struct Orders {
    rows: Vec<Row>,
    /// The indexes of the rows that pass the filter.
    visible: Vec<usize>,
}

impl Orders {
    pub fn new(count: usize) -> Self {
        Self {
            rows: spec::orders(count).into_iter().map(Row::new).collect(),
            visible: (0..count).collect(),
        }
    }

    /// Shows the orders whose status is shown (`shown[k]` for `Status::ALL[k]`) and whose
    /// customer holds `search`.
    pub fn filter(&mut self, shown: &[bool; 4], search: &str, cx: &mut Context<Self>) {
        let visible: Vec<usize> = (0..self.rows.len())
            .filter(|&i| spec::visible(&self.rows[i].order, shown, search))
            .collect();
        if visible != self.visible {
            self.visible = visible;
            cx.notify();
        }
    }
}

/// Returns the header or a row: a 1 px bottom border and the five columns, the last one right
/// aligned.
fn columns(height: f32, cells: [AnyElement; 4], amount: impl IntoElement) -> Div {
    h_flex()
        .flex_none()
        .w_full()
        .h(px(height))
        .pl(px(spec::COL_ORDER))
        .pr(px(AMOUNT_PADDING))
        .border_b_1()
        .overflow_hidden()
        .children(
            cells
                .into_iter()
                .zip([
                    ORDER_WIDTH,
                    CUSTOMER_WIDTH,
                    STATUS_WIDTH,
                    spec::PROGRESS_WIDTH,
                ])
                .map(|(cell, width)| h_flex().flex_none().w(px(width)).child(cell)),
        )
        .child(h_flex().flex_1().min_w_0().justify_end().child(amount))
}

/// Returns the header with the column titles.
fn header() -> Div {
    let [order, customer, status, progress, amount] = spec::COLUMNS.map(|title| div().child(title));
    columns(
        spec::TABLE_HEADER_HEIGHT,
        [
            order.into_any_element(),
            customer.into_any_element(),
            status.into_any_element(),
            progress.into_any_element(),
        ],
        amount,
    )
    .border_color(rgb(spec::RULE))
    .rounded_t(px(spec::CARD_RADIUS))
    .bg(rgb(spec::TABLE_HEADER_BG))
    .text_size(px(12.0))
    .font_weight(FontWeight::SEMIBOLD)
    .text_color(rgb(spec::MUTED))
}

/// Returns the row of order `row` at index `i`, with the hover color. The last row rounds its
/// bottom corners like the card.
fn row(i: usize, row: &Row, last: bool) -> Stateful<Div> {
    let (pill_bg, pill_text) = row.order.status.colors();
    let customer = h_flex()
        .gap(px(NAME_OFFSET - spec::AVATAR))
        .child(
            h_flex()
                .flex_none()
                .size(px(spec::AVATAR))
                .justify_center()
                .rounded_full()
                .bg(rgb(row.order.avatar))
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xffffff))
                .child(row.initials.clone()),
        )
        .child(row.customer.clone());
    let status = h_flex()
        .h(px(22.0))
        .px(px(10.0))
        .rounded(px(11.0))
        .bg(rgb(pill_bg))
        .text_size(px(12.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(pill_text))
        .child(row.order.status.label());
    let progress = div()
        .w(px(spec::PROGRESS_WIDTH))
        .h(px(spec::PROGRESS_HEIGHT))
        .rounded(px(spec::PROGRESS_HEIGHT / 2.0))
        .bg(rgb(spec::TRACK))
        .child(
            div()
                .h_full()
                .w(px(spec::PROGRESS_WIDTH * row.order.progress as f32 / 100.0))
                .rounded(px(spec::PROGRESS_HEIGHT / 2.0))
                .bg(rgb(spec::PRIMARY)),
        );
    columns(
        spec::ORDER_ROW_HEIGHT,
        [
            div()
                .text_color(rgb(spec::MUTED))
                .child(row.number.clone())
                .into_any_element(),
            customer.into_any_element(),
            status.into_any_element(),
            progress.into_any_element(),
        ],
        div()
            .font_weight(FontWeight::SEMIBOLD)
            .child(row.amount.clone()),
    )
    .id(i)
    .border_color(rgb(spec::ROW_RULE))
    .bg(rgb(spec::CARD_BG))
    .hover(|style| style.bg(rgb(spec::ROW_HOVER)))
    .when(last, |row| row.rounded_b(px(spec::CARD_RADIUS)))
}

impl Render for Orders {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let count = self.visible.len();
        let rows = self
            .visible
            .iter()
            .enumerate()
            .map(|(at, &i)| row(i, &self.rows[i], at + 1 == count));
        card()
            .flex_none()
            .w_full()
            .child(v_flex().w_full().child(header()).children(rows))
    }
}
