//! The orders card: its own view that holds every order, observes the filters and the search,
//! and shows the header and one real row per visible order, each with an avatar, a status pill
//! and a progress bar. The list scenario shows the same header and rows.

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::filters::Filters;
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

/// One order with its texts.
pub struct Row {
    order: Order,
    number: SharedString,
    customer: SharedString,
    initials: SharedString,
    amount: SharedString,
}

impl Row {
    pub fn new(order: Order) -> Self {
        Self {
            number: order.number_text().into(),
            customer: order.customer.clone().into(),
            initials: order.initials.clone().into(),
            amount: order.amount().into(),
            order,
        }
    }
}

/// The orders, and the ones the filters show.
pub struct Orders {
    /// Every order, made once.
    rows: Vec<Row>,
    /// The indexes of the rows that pass the filters, in the order the table shows them.
    visible: Vec<usize>,
    /// Whether the rows are sorted by amount.
    sorted: bool,
    filters: Entity<Filters>,
    search: Entity<InputState>,
    _changes: [Subscription; 2],
}

impl Orders {
    pub fn new(
        count: usize,
        filters: Entity<Filters>,
        search: Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> Self {
        let changes = [
            cx.observe(&filters, |this, _, cx| this.filter(cx)),
            cx.subscribe(&search, |this, _, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    this.filter(cx);
                }
            }),
        ];
        let mut orders = Self {
            rows: spec::orders(count).into_iter().map(Row::new).collect(),
            visible: Vec::new(),
            sorted: false,
            filters,
            search,
            _changes: changes,
        };
        orders.filter(cx);
        orders
    }

    /// Shows the orders whose status is checked and whose customer holds the search text, by
    /// amount when the filters say so (the order of `spec::arranged`).
    fn filter(&mut self, cx: &mut Context<Self>) {
        let search = self.search.read(cx).value();
        let filters = self.filters.read(cx);
        let mut visible: Vec<usize> = (0..self.rows.len())
            .filter(|&i| spec::visible(&self.rows[i].order, &filters.shown, &search))
            .collect();
        let sorted = filters.sorted;
        if sorted {
            visible.sort_by_key(|&i| std::cmp::Reverse(self.rows[i].order.cents));
        }
        if visible != self.visible || sorted != self.sorted {
            self.visible = visible;
            self.sorted = sorted;
            cx.notify();
        }
    }

    /// Returns the height of the card: the header and the visible rows.
    pub fn height(&self) -> f32 {
        spec::TABLE_HEADER_HEIGHT + spec::ORDER_ROW_HEIGHT * self.visible.len() as f32
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
        .child(
            h_flex()
                .flex_1()
                .min_w_0()
                .h_full()
                .justify_end()
                .child(amount),
        )
}

/// Returns the header with the column titles. The amount title is `PRIMARY` while `sorted`, and
/// a click on it calls `on_sort`.
pub fn header(
    sorted: bool,
    on_sort: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let [order, customer, status, progress] =
        [0, 1, 2, 3].map(|k| div().child(spec::COLUMNS[k]).into_any_element());
    let amount = h_flex()
        .id("sort")
        .w(px(spec::SORT_WIDTH))
        .h_full()
        .justify_end()
        .when(sorted, |title| title.text_color(rgb(spec::PRIMARY)))
        .on_click(on_sort)
        .child(spec::COLUMNS[spec::SORT_COLUMN]);
    columns(
        spec::TABLE_HEADER_HEIGHT,
        [order, customer, status, progress],
        amount,
    )
    .border_color(rgb(spec::RULE))
    .bg(rgb(spec::TABLE_HEADER_BG))
    .text_size(px(12.0))
    .font_weight(FontWeight::SEMIBOLD)
    .text_color(rgb(spec::MUTED))
}

/// Returns the row of `row` with the element id `id` and the hover color.
pub fn row(id: usize, row: &Row) -> Stateful<Div> {
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
    .id(id)
    .border_color(rgb(spec::ROW_RULE))
    .bg(rgb(spec::CARD_BG))
    .hover(|style| style.bg(rgb(spec::ROW_HOVER)))
}

impl Render for Orders {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let filters = self.filters.clone();
        let header = header(self.sorted, move |_, _, cx| {
            filters.update(cx, |filters, cx| filters.toggle_sorted(cx))
        })
        .rounded_t(px(spec::CARD_RADIUS));
        // The last row rounds its bottom corners like the card.
        let count = self.visible.len();
        let rows = self.visible.iter().enumerate().map(|(at, &i)| {
            row(i, &self.rows[i]).when(at + 1 == count, |row| row.rounded_b(px(spec::CARD_RADIUS)))
        });
        card()
            .flex_none()
            .w_full()
            .child(v_flex().w_full().child(header).children(rows))
    }
}
