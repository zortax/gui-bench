//! The orders card: a header with the column titles and a keyed list of order row components,
//! one for every order that passes the status filter and the search. All rows are real
//! elements. A row holds its own hover state, so a hover change renders only that row.

use std::rc::Rc;

use freya::prelude::*;

use super::{avatar, card, pill, spec, text};
use crate::page::{bottom_border, color};

/// The column boxes from the left of the card: the order number, the customer, the status and
/// the progress bar; the amount takes the rest and ends `COL_ORDER` inside the card.
const COLUMNS: [f32; 4] = [
    spec::COL_CUSTOMER - spec::COL_ORDER,
    spec::COL_STATUS - spec::COL_CUSTOMER,
    spec::COL_PROGRESS - spec::COL_STATUS,
    spec::PROGRESS_WIDTH,
];

/// The orders card. It reads the status filter and the search, so a change renders the list,
/// and the keyed diff keeps every row component that stays.
#[derive(PartialEq)]
pub struct Orders {
    pub orders: Rc<Vec<Rc<spec::Order>>>,
    pub shown: State<[bool; 4]>,
    pub search: State<String>,
}

impl Component for Orders {
    fn render(&self) -> impl IntoElement {
        let shown = *self.shown.read();
        let search = self.search.read();
        let rows: Vec<Element> = self
            .orders
            .iter()
            .filter(|order| spec::visible(order, &shown, &search))
            .map(|order| {
                OrderRow {
                    order: order.clone(),
                    key: DiffKey::None,
                }
                .key(order.number)
                .into_element()
            })
            .collect();

        card()
            .width(Size::fill())
            .overflow(Overflow::Clip)
            .child(header())
            .children(rows)
    }
}

/// Returns a row of `height` with the five columns, centered above its 1 px bottom border.
fn columns(height: f32, cells: [Element; 5]) -> Rect {
    let [order, customer, status, progress, amount] = cells;
    rect()
        .horizontal()
        .content(Content::Flex)
        .width(Size::fill())
        .height(Size::px(height))
        .padding(Gaps::new(0.0, spec::COL_ORDER, 1.0, spec::COL_ORDER))
        .cross_align(Alignment::Center)
        .child(rect().width(Size::px(COLUMNS[0])).child(order))
        .child(rect().width(Size::px(COLUMNS[1])).child(customer))
        .child(rect().width(Size::px(COLUMNS[2])).child(status))
        .child(rect().width(Size::px(COLUMNS[3])).child(progress))
        .child(
            rect()
                .horizontal()
                .width(Size::flex(1.0))
                .main_align(Alignment::End)
                .child(amount),
        )
}

/// Returns the header with the column titles, rounded at the top like the card.
fn header() -> Rect {
    let [order, customer, status, progress, amount] =
        spec::COLUMNS.map(|title| text(title, 12.0, FontWeight::SEMI_BOLD, spec::MUTED).into());
    columns(
        spec::TABLE_HEADER_HEIGHT,
        [order, customer, status, progress, amount],
    )
    .background(color(spec::TABLE_HEADER_BG))
    .corner_radius(CornerRadius::new(
        spec::CARD_RADIUS,
        spec::CARD_RADIUS,
        0.0,
        0.0,
    ))
    .border(bottom_border(spec::RULE))
}

/// One order row. Its props are the order, so a list render keeps it as it is.
#[derive(PartialEq)]
struct OrderRow {
    order: Rc<spec::Order>,
    key: DiffKey,
}

impl KeyExt for OrderRow {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for OrderRow {
    fn render(&self) -> impl IntoElement {
        let mut hovered = use_state(|| false);
        let order = &self.order;
        let background = if *hovered.read() {
            spec::ROW_HOVER
        } else {
            spec::CARD_BG
        };
        let customer = rect()
            .horizontal()
            .spacing(40.0 - spec::AVATAR)
            .cross_align(Alignment::Center)
            .child(avatar(
                order.initials.clone(),
                spec::AVATAR,
                order.avatar,
                11.0,
            ))
            .child(text(
                order.customer.clone(),
                spec::FONT_SIZE,
                FontWeight::NORMAL,
                spec::TEXT,
            ));
        let progress = rect()
            .width(Size::px(spec::PROGRESS_WIDTH))
            .height(Size::px(spec::PROGRESS_HEIGHT))
            .corner_radius(spec::PROGRESS_HEIGHT / 2.0)
            .background(color(spec::TRACK))
            .child(
                rect()
                    .width(Size::px(
                        spec::PROGRESS_WIDTH * order.progress as f32 / 100.0,
                    ))
                    .height(Size::fill())
                    .corner_radius(spec::PROGRESS_HEIGHT / 2.0)
                    .background(color(spec::PRIMARY)),
            );

        columns(
            spec::ORDER_ROW_HEIGHT,
            [
                text(
                    order.number_text(),
                    spec::FONT_SIZE,
                    FontWeight::NORMAL,
                    spec::MUTED,
                )
                .into(),
                customer.into(),
                pill(order.status.label(), 22.0, 10.0, order.status.colors()).into(),
                progress.into(),
                text(
                    order.amount(),
                    spec::FONT_SIZE,
                    FontWeight::SEMI_BOLD,
                    spec::TEXT,
                )
                .into(),
            ],
        )
        .background(color(background))
        .border(bottom_border(spec::ROW_RULE))
        .on_pointer_enter(move |_| hovered.set_if_modified(true))
        .on_pointer_leave(move |_| hovered.set_if_modified(false))
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
