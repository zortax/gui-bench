//! The dashboard scenario: a sidebar, a top bar and one scroll container with the metric cards,
//! the chart and filter cards and the orders table. One state struct, one `Message` enum, and a
//! `view` that builds the whole page from the state, the Elm architecture of iced.

mod chart;
mod filters;
mod kpis;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
mod spec;
mod style;
mod topbar;

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{column, row, scrollable};
use iced::{Element, Length, Theme, theme};

use crate::page::{self, color};
use spec::{Order, Status};

/// The scrollbar floats over the right padding of the content, so it takes no layout space.
const SCROLLBAR_WIDTH: f32 = 6.0;
const SCROLLBAR_MARGIN: f32 = 4.0;

/// An order with its texts formatted once.
struct OrderRow {
    order: Order,
    number: String,
    amount: String,
}

/// The page state.
struct Dashboard {
    orders: Vec<OrderRow>,
    /// The active nav item.
    nav: usize,
    /// The text of the search input.
    search: String,
    /// The selected segment of the chart.
    segment: usize,
    /// The compare switch.
    compare: bool,
    /// The status checkboxes, in the order of `Status::ALL`.
    shown: [bool; 4],
}

#[derive(Debug, Clone)]
enum Message {
    Nav(usize),
    Search(String),
    Export,
    Segment(usize),
    Shown(Status, bool),
    Compare(bool),
    Reset,
    Apply,
    /// A press on an order row, which has no action.
    Order,
}

/// Opens the window and runs the event loop until the window closes.
pub fn run(count: usize, title: &'static str, app_id: &str) -> iced::Result {
    iced::application(
        move || Dashboard::new(count),
        Dashboard::update,
        Dashboard::view,
    )
    .title(title)
    .theme(Theme::Light)
    .style(|_, _| theme::Style {
        background_color: color(spec::PAGE_BG),
        text_color: color(spec::TEXT),
    })
    .window(page::window(app_id))
    .run()
}

impl Dashboard {
    fn new(count: usize) -> Self {
        let orders = spec::orders(count)
            .into_iter()
            .map(|order| OrderRow {
                number: order.number_text(),
                amount: order.amount(),
                order,
            })
            .collect();
        Self {
            orders,
            nav: 0,
            search: String::new(),
            segment: 0,
            compare: false,
            shown: [true; 4],
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Nav(item) => self.nav = item,
            Message::Search(search) => self.search = search,
            Message::Segment(segment) => self.segment = segment,
            Message::Shown(status, shown) => self.shown[status.filter_row()] = shown,
            Message::Compare(compare) => self.compare = compare,
            Message::Reset => {
                self.shown = [true; 4];
                self.compare = false;
                self.segment = 0;
                self.search.clear();
            }
            Message::Export | Message::Apply | Message::Order => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let shown = self
            .orders
            .iter()
            .filter(|row| spec::visible(&row.order, &self.shown, &self.search));
        let content = column![
            kpis::view(),
            row![
                chart::view(self.segment, self.compare),
                filters::view(&self.shown, self.compare),
            ]
            .spacing(spec::GAP)
            .height(spec::ROW2_HEIGHT),
            orders::view(shown),
        ]
        .spacing(spec::GAP)
        .padding(spec::CONTENT_PADDING)
        .width(Length::Fill);
        let scrollbar = Scrollbar::new()
            .width(SCROLLBAR_WIDTH)
            .scroller_width(SCROLLBAR_WIDTH)
            .margin(SCROLLBAR_MARGIN);
        let scroll = scrollable(content)
            .direction(Direction::Vertical(scrollbar))
            .width(Length::Fill)
            .height(Length::Fill);
        row![
            sidebar::view(self.nav),
            column![topbar::view(&self.search), scroll],
        ]
        .into()
    }
}
