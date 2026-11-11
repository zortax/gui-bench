//! The table scenario: a header with the row count above a table of rows. The function keys run
//! the table operations of `spec::Op`, a click selects a row, and with a ticker a timer updates
//! rows and the header. Each variant holds and views the rows its own way.

mod coarse;
mod fine;
mod keys;

use std::time::Duration;

use iced::widget::mouse_area;
use iced::{Element, Subscription, Theme};

use crate::cli::{Ticker, Variant};
use crate::page;
use crate::spec::{self, Op};

/// The messages of the table.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    /// A function key named a table operation.
    Key(Op),
    /// A row was clicked.
    Select(u64),
    /// The ticker timer fired.
    Tick,
}

/// Opens the window of the table in `variant` and runs it until the window closes.
pub fn run(
    count: usize,
    variant: Variant,
    ticker: Option<Ticker>,
    title: &'static str,
    app_id: &str,
) -> iced::Result {
    match variant {
        Variant::Coarse => iced::application(
            move || coarse::Table::new(count, ticker),
            coarse::Table::update,
            coarse::Table::view,
        )
        .subscription(|table| subscription(table.ticker))
        .title(title)
        .theme(Theme::Light)
        .style(page::style)
        .window(page::window(app_id))
        .run(),
        _ => iced::application(
            move || fine::Table::new(count, ticker),
            fine::Table::update,
            fine::Table::view,
        )
        .subscription(|table| subscription(table.ticker))
        .title(title)
        .theme(Theme::Light)
        .style(page::style)
        .window(page::window(app_id))
        .run(),
    }
}

/// Returns the function keys, and the timer when the table has a ticker.
fn subscription(ticker: Option<Ticker>) -> Subscription<Message> {
    let keys = keys::listen();
    match ticker {
        Some(ticker) => Subscription::batch([
            keys,
            iced::time::every(Duration::from_millis(ticker.interval_ms)).map(|_| Message::Tick),
        ]),
        None => keys,
    }
}

/// Returns the header status of a table with `len` rows.
fn status(len: usize, ticker: Option<Ticker>, tick: u64) -> String {
    match ticker {
        Some(_) => spec::ticker_status(len, tick),
        None => spec::status(len),
    }
}

/// Returns the new selection after a click on row `id`: the row, or none when it was selected.
fn toggle(selected: Option<u64>, id: u64) -> Option<u64> {
    if selected == Some(id) { None } else { Some(id) }
}

/// Builds one row: a press selects it. The row has no hover color.
fn table_row(id: u64, label: &str, value: u32, selected: bool) -> Element<'static, Message> {
    let background = if selected {
        spec::SELECTED
    } else {
        spec::ROW_EVEN
    };
    mouse_area(page::row(label, value, background))
        .on_press(Message::Select(id))
        .into()
}
