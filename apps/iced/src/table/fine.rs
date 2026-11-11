//! The fine variant: a keyed column in which every row is a `lazy` widget over the row id, a
//! per-row version and whether the row is selected. The model is still one list, but each row
//! carries a version that every edit of the row bumps, so `view` rebuilds only the rows whose
//! dependency hash changed (the edited rows, and the old and new selected row); iced keeps the
//! cached widgets of every other row, and the keyed column keeps them with their id across moves,
//! inserts and removals.

use iced::widget::{column, keyed_column, lazy, scrollable, space};
use iced::{Element, Length};

use super::{Message, status, table_row, toggle};
use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids, Row, TableRow};

/// A row with the version that every edit of it bumps.
struct FineRow {
    row: Row,
    version: u64,
}

impl TableRow for FineRow {
    fn from_row(row: Row) -> Self {
        Self { row, version: 0 }
    }

    fn toggle_suffix(&mut self) {
        self.row.toggle_suffix();
        self.version += 1;
    }
}

/// The table of versioned rows.
pub struct Table {
    count: usize,
    pub ticker: Option<Ticker>,
    ids: Ids,
    rows: Vec<FineRow>,
    selected: Option<u64>,
    tick: u64,
}

impl Table {
    /// Returns the table with `count` first rows.
    pub fn new(count: usize, ticker: Option<Ticker>) -> Self {
        let mut ids = Ids::default();
        let rows = ids.rows_of(count);
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
                    let fine = &mut self.rows[at];
                    fine.row.value = spec::ticker_value(fine.row.id, self.tick);
                    fine.version += 1;
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let rows = keyed_column(self.rows.iter().map(|fine| {
            let id = fine.row.id;
            let selected = self.selected == Some(id);
            let row = lazy((id, fine.version, selected), move |_| {
                table_row(id, &fine.row.label, fine.row.value, selected)
            });
            (id, row.into())
        }))
        .width(Length::Fill);
        column![
            page::header(status(self.rows.len(), self.ticker, self.tick), space()),
            page::list(scrollable(rows)),
        ]
        .into()
    }
}
