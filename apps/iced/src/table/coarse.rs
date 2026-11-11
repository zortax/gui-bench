//! The coarse variant, iced's default: the model holds plain rows, and every message (an edit, a
//! tick or a new selection) runs `view` again, which builds a column of every row, and iced diffs
//! the whole new tree against the old one.

use iced::widget::{Column, column, scrollable, space};
use iced::{Element, Length};

use super::{Message, status, table_row, toggle};
use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids, Row};

/// The table with one list of plain rows.
pub struct Table {
    count: usize,
    pub ticker: Option<Ticker>,
    ids: Ids,
    rows: Vec<Row>,
    selected: Option<u64>,
    tick: u64,
}

impl Table {
    /// Returns the table with `count` first rows.
    pub fn new(count: usize, ticker: Option<Ticker>) -> Self {
        let mut ids = Ids::default();
        let rows = ids.rows(count);
        Self {
            count,
            ticker,
            ids,
            rows,
            selected: None,
            tick: 0,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Key(op) => op.apply(&mut self.rows, &mut self.ids, self.count),
            Message::Select(id) => self.selected = toggle(self.selected, id),
            Message::Tick => {
                let Some(ticker) = self.ticker else { return };
                self.tick += 1;
                for at in spec::ticker_positions(self.rows.len(), ticker.updates) {
                    let row = &mut self.rows[at];
                    row.value = spec::ticker_value(row.id, self.tick);
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let rows =
            Column::with_children(self.rows.iter().map(|row| {
                table_row(row.id, &row.label, row.value, self.selected == Some(row.id))
            }))
            .width(Length::Fill);
        column![
            page::header(status(self.rows.len(), self.ticker, self.tick), space()),
            page::list(scrollable(rows)),
        ]
        .into()
    }
}
