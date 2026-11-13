//! The list scenario: the header of the orders over a list of order rows of which only the
//! visible ones are built. iced has no virtual list, so this one is made by hand in the way its
//! `Scrollable` allows: `on_scroll` reports the viewport, and the view builds the rows in and
//! near it between two fixed-height spaces that stand in for the rows above and below.

use std::borrow::Cow;

use iced::widget::scrollable::{Direction, Viewport};
use iced::widget::{column, container, scrollable, space};
use iced::{Element, Length, Theme};

use super::style;
use super::{orders, spec};

/// The rows built past each edge of the viewport.
const OVERSCAN: usize = 4;

/// The list state.
struct List {
    /// The index of the order of every row, top to bottom.
    order: Vec<u32>,
    /// Whether the orders are sorted by amount.
    sorted: bool,
    /// The visible part of the rows: its top and its height.
    offset: f32,
    height: f32,
}

#[derive(Debug, Clone)]
enum Message {
    /// A click on the amount title.
    Sort,
    /// The list scrolled or changed size.
    Scrolled(Viewport),
    /// A press on an order row, which has no action.
    Order,
}

/// Opens the window and runs the event loop until the window closes.
pub fn run(count: usize, title: &'static str, app_id: &str) -> iced::Result {
    let boot = move || List {
        order: spec::list_order(count, false),
        sorted: false,
        offset: 0.0,
        height: spec::WINDOW.1 - spec::LIST_ROWS_TOP,
    };
    iced::application(boot, List::update, List::view)
        .title(title)
        .theme(Theme::Light)
        .style(super::app_style)
        .window(super::window(app_id))
        .run()
}

impl List {
    fn update(&mut self, message: Message) {
        match message {
            Message::Sort => {
                self.sorted = !self.sorted;
                self.order = spec::list_order(self.order.len(), self.sorted);
            }
            Message::Scrolled(viewport) => {
                self.offset = viewport.absolute_offset().y;
                self.height = viewport.bounds().height;
            }
            Message::Order => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let count = self.order.len();
        let first = ((self.offset / spec::ORDER_ROW_HEIGHT) as usize)
            .saturating_sub(OVERSCAN)
            .min(count);
        let last = (((self.offset + self.height) / spec::ORDER_ROW_HEIGHT).ceil() as usize
            + OVERSCAN)
            .min(count);
        let rows = column![
            space().height(first as f32 * spec::ORDER_ROW_HEIGHT),
            rows(&self.order[first..last]),
            space().height((count - last) as f32 * spec::ORDER_ROW_HEIGHT),
        ]
        .width(Length::Fill);
        let list = scrollable(rows)
            .direction(Direction::Vertical(super::scrollbar()))
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll(Message::Scrolled);
        container(column![
            orders::header(self.sorted, Message::Sort, 0.0),
            list
        ])
        .style(|_| style::filled(spec::CARD_BG, 0))
        .into()
    }
}

/// Returns the rows of the orders `indices`, each order made here. The direct edition builds
/// them on each view; the tuned edition wraps them in iced's `lazy`, which rebuilds them only
/// when the orders in view change.
fn rows(indices: &[u32]) -> Element<'_, Message> {
    let build = |indices: &[u32]| -> Element<'static, Message> {
        column(indices.iter().map(|&index| {
            let order = spec::order(index as usize);
            orders::order_row(Cow::Owned(order), false, Message::Order)
        }))
        .into()
    };
    #[cfg(feature = "tuned")]
    return iced::widget::lazy(indices, move |indices| build(indices)).into();
    #[cfg(not(feature = "tuned"))]
    build(indices)
}
