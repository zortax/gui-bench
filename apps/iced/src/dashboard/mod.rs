//! The app scenario: a sidebar with nav, a top bar with the search, the live switch and the
//! export menu, and one scroll container with the page of the active nav item: the overview (the
//! metric cards, the revenue chart, the filters and the orders table), the messages feed, or an
//! empty page (`dashboard.rs`). One state struct, one `Message` enum, and a `view` that builds
//! the whole window from the state, the Elm architecture of iced.

mod chart;
mod filters;
mod kpis;
pub mod list;
mod live;
mod menu;
mod messages;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
mod spec;
mod style;
mod topbar;

use std::time::Instant;

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{column, container, row, scrollable, stack, text};
use iced::{Element, Length, Size, Theme, padding, theme, window};

use live::Live;
use spec::{Order, Page, Status};
use style::color;

/// The scrollbar floats over the right padding of the content, so it takes no layout space.
const SCROLLBAR_WIDTH: f32 = 6.0;
const SCROLLBAR_MARGIN: f32 = 4.0;
/// The top of the label of the empty page inside the padding of the content.
const EMPTY_LABEL_TOP: f32 = spec::EMPTY_TOP - spec::CONTENT_PADDING;

/// The app state.
struct Dashboard {
    /// The orders in the order the table shows them.
    orders: Vec<Order>,
    /// The messages of the feed.
    messages: Vec<spec::Message>,
    /// The active nav item, which picks the page.
    nav: usize,
    /// The text of the search input.
    search: String,
    /// The selected segment of the chart.
    segment: usize,
    /// The compare switch.
    compare: bool,
    /// The status checkboxes, in the order of `Status::ALL`.
    shown: [bool; 4],
    /// Whether the orders are sorted by amount.
    sorted: bool,
    /// Whether the export menu is open.
    menu: bool,
    /// The live switch, the tick of the metric values and the spinner.
    live: Live,
}

#[derive(Debug, Clone)]
enum Message {
    Nav(usize),
    Search(String),
    /// A click on the export button.
    Export,
    /// A click on an item of the export menu.
    MenuItem,
    Segment(usize),
    Shown(Status, bool),
    Compare(bool),
    /// A click on the amount title of the orders.
    Sort,
    Reset,
    Apply,
    /// A press on an order row, which has no action.
    Order,
    /// The live switch.
    Live(bool),
    /// The timer of the live updates.
    Tick,
    /// A presented frame while the spinner turns.
    Frame(Instant),
}

/// Opens the window and runs the event loop until the window closes.
pub fn run(count: usize, title: &'static str, app_id: &str) -> iced::Result {
    iced::application(
        move || Dashboard::new(count),
        Dashboard::update,
        Dashboard::view,
    )
    .subscription(|dashboard| dashboard.live.subscription())
    .title(title)
    .theme(Theme::Light)
    .style(app_style)
    .window(window(app_id))
    .run()
}

/// Returns the undecorated window of both scenarios, in the size of the spec.
fn window(app_id: &str) -> window::Settings {
    window::Settings {
        size: Size::new(spec::WINDOW.0, spec::WINDOW.1),
        decorations: false,
        platform_specific: window::settings::PlatformSpecific {
            application_id: app_id.to_owned(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The application style: the page background and the text color of the spec.
fn app_style<State>(_: &State, _: &Theme) -> theme::Style {
    theme::Style {
        background_color: color(spec::PAGE_BG),
        text_color: color(spec::TEXT),
    }
}

/// Returns the vertical scrollbar of the scroll containers: a 6 px bar that floats over the
/// content.
fn scrollbar() -> Scrollbar {
    Scrollbar::new()
        .width(SCROLLBAR_WIDTH)
        .scroller_width(SCROLLBAR_WIDTH)
        .margin(SCROLLBAR_MARGIN)
}

impl Dashboard {
    fn new(count: usize) -> Self {
        Self {
            orders: spec::orders(count),
            messages: spec::messages(count),
            nav: 0,
            search: String::new(),
            segment: 0,
            compare: false,
            shown: [true; 4],
            sorted: false,
            menu: false,
            live: Live::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Nav(item) => self.nav = item,
            Message::Search(search) => self.search = search,
            Message::Export => self.menu = !self.menu,
            Message::MenuItem => self.menu = false,
            Message::Segment(segment) => self.segment = segment,
            Message::Shown(status, shown) => self.shown[status.filter_row()] = shown,
            Message::Compare(compare) => self.compare = compare,
            Message::Sort => self.sort(!self.sorted),
            Message::Reset => {
                self.shown = [true; 4];
                self.compare = false;
                self.segment = 0;
                self.search.clear();
                self.sort(false);
            }
            Message::Live(on) => self.live.set(on),
            Message::Tick => self.live.tick += 1,
            Message::Frame(now) => self.live.frame(now),
            Message::Apply | Message::Order => {}
        }
    }

    /// Puts the orders in the order of the data, or by amount when `sorted`.
    fn sort(&mut self, sorted: bool) {
        if sorted != self.sorted {
            self.sorted = sorted;
            self.orders = spec::arranged(spec::orders(self.orders.len()), sorted);
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let scroll = scrollable(
            container(self.page())
                .padding(spec::CONTENT_PADDING)
                .width(Length::Fill),
        )
        .direction(Direction::Vertical(scrollbar()))
        .width(Length::Fill)
        .height(Length::Fill);
        let title = spec::NAV_ITEMS[self.nav];
        let window = row![
            sidebar::view(self.nav),
            column![topbar::view(title, &self.search, &self.live), scroll],
        ];
        if self.menu {
            stack![window, menu::view()].into()
        } else {
            window.into()
        }
    }

    /// Returns the page of the active nav item.
    fn page(&self) -> Element<'_, Message> {
        match spec::page(self.nav) {
            Page::Overview => column![
                kpis::view(self.live.tick),
                row![
                    chart::view(self.segment, self.compare),
                    filters::view(&self.shown, self.compare),
                ]
                .spacing(spec::GAP)
                .height(spec::ROW2_HEIGHT),
                orders::view(&self.orders, self.shown, &self.search, self.sorted),
            ]
            .spacing(spec::GAP)
            .into(),
            Page::Messages => messages::view(&self.messages),
            Page::Empty => container(
                text(spec::EMPTY_LABEL)
                    .size(spec::FONT_SIZE)
                    .color(color(spec::MUTED)),
            )
            .center_x(Length::Fill)
            .padding(padding::top(EMPTY_LABEL_TOP))
            .into(),
        }
    }
}
