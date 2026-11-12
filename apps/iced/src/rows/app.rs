//! The rows application: the undecorated light window and the view, a vertical scroll container
//! over a column of every row.

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Column, scrollable};
use iced::{Element, Length, Size, Theme, theme, window};

use super::row::{self, Message};

const TITLE: &str = "gui-bench rows";
const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 800.0;

/// The scrollbar takes `WIDTH + 2 * MARGIN + SPACING` at the right edge.
const SCROLLBAR_WIDTH: f32 = 6.0;
const SCROLLBAR_MARGIN: f32 = 4.0;
const SCROLLBAR_SPACING: f32 = 1.0;

/// The application state: the row count.
struct Rows {
    count: usize,
}

/// Opens the window and runs the event loop until the window closes.
pub fn run(count: usize, app_id: &str) -> iced::Result {
    let window = window::Settings {
        size: Size::new(WIDTH, HEIGHT),
        decorations: false,
        platform_specific: window::settings::PlatformSpecific {
            application_id: app_id.to_owned(),
            ..Default::default()
        },
        ..Default::default()
    };
    iced::application(move || Rows { count }, update, view)
        .title(TITLE)
        .theme(Theme::Light)
        .style(|_, _| theme::Style {
            background_color: row::BACKGROUND,
            text_color: row::LABEL,
        })
        .window(window)
        .run()
}

/// Ignores row presses: the rows have no action.
fn update(_: &mut Rows, _: Message) {}

/// Returns the vertical scrollbar of every scroll container: a 6 px bar in its own gutter.
pub fn scrollbar() -> Scrollbar {
    Scrollbar::new()
        .width(SCROLLBAR_WIDTH)
        .scroller_width(SCROLLBAR_WIDTH)
        .margin(SCROLLBAR_MARGIN)
        .spacing(SCROLLBAR_SPACING)
}

fn view(rows: &Rows) -> Element<'_, Message> {
    let list = Column::with_children((0..rows.count).map(row::row)).width(Length::Fill);
    scrollable(list)
        .direction(Direction::Vertical(scrollbar()))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
