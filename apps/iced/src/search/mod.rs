//! The search and typing scenarios: a header with iced's text input, focused at startup by a
//! widget operation that the boot task runs, above `count` rows. With `filter`, every edit shows
//! the rows whose label holds the text in a column keyed by row id; without, the rows stay.

use iced::widget::{column, keyed_column, operation, scrollable, text_input};
use iced::{Element, Length, Task, Theme};

use crate::page;
use crate::spec::{self, Ids, Row};

/// The widget id of the text input.
const INPUT: &str = "search";

/// The page state: every row and the text of the input.
struct Search {
    rows: Vec<Row>,
    query: String,
    filter: bool,
}

/// The message of an edit of the input.
#[derive(Debug, Clone)]
enum Message {
    Query(String),
}

/// Opens the window and runs the event loop until the window closes.
pub fn run(count: usize, filter: bool, title: &'static str, app_id: &str) -> iced::Result {
    iced::application(
        move || Search::new(count, filter),
        Search::update,
        Search::view,
    )
    .title(title)
    .theme(Theme::Light)
    .style(page::style)
    .window(page::window(app_id))
    .run()
}

impl Search {
    /// Returns the state and the task that focuses the input.
    fn new(count: usize, filter: bool) -> (Self, Task<Message>) {
        let search = Self {
            rows: Ids::default().rows(count),
            query: String::new(),
            filter,
        };
        (search, operation::focus(INPUT))
    }

    fn update(&mut self, message: Message) {
        let Message::Query(query) = message;
        self.query = query;
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search", &self.query)
            .id(INPUT)
            .on_input(Message::Query)
            .size(spec::FONT_SIZE)
            .width(spec::INPUT_WIDTH);
        let shown = self
            .rows
            .iter()
            .filter(|row| !self.filter || spec::matches(&row.label, &self.query));
        let rows =
            keyed_column(shown.map(|row| (row.id, page::plain_row(row)))).width(Length::Fill);
        column![
            page::header(spec::status(self.rows.len()), input),
            page::list(scrollable(rows)),
        ]
        .into()
    }
}
