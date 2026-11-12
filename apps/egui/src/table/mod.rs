//! The table and ticker scenarios: a header with the row count above all rows of a `Vec<Row>`.
//! The function keys run the table operations of `spec::Op`, a click selects a row, and with a
//! ticker the rows and the header change at every interval.
//!
//! egui is immediate mode, so there is one variant only (`default`, the immediate one): the app
//! owns plain data and every pass lays out the whole page from it. There is no per-row state to
//! notify; an edit changes the data and the next pass shows it.

mod keys;
mod ticker;

use eframe::egui::{self, Context, Ui};

use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids, Row};
use crate::window;

/// Opens the table window with `count` rows, and a ticker when `ticker` is set.
pub fn run(title: &str, count: usize, ticker: Option<Ticker>) -> eframe::Result {
    window::run(title, page::setup, move || {
        Box::new(Table::new(count, ticker))
    })
}

/// The table data and its selection.
struct Table {
    count: usize,
    ids: Ids,
    rows: Vec<Row>,
    /// The id of the selected row.
    selected: Option<u64>,
    ticker: Option<ticker::Clock>,
}

impl Table {
    fn new(count: usize, ticker: Option<Ticker>) -> Self {
        let mut ids = Ids::default();
        let rows = ids.rows(count);
        Self {
            count,
            ids,
            rows,
            selected: None,
            ticker: ticker.map(ticker::Clock::new),
        }
    }

    /// Selects the row with `id`, or clears the selection when it is the selected one.
    fn toggle_selection(&mut self, id: u64) {
        self.selected = if self.selected == Some(id) {
            None
        } else {
            Some(id)
        };
    }
}

impl eframe::App for Table {
    fn logic(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        if let Some(clock) = &mut self.ticker {
            clock.advance(ctx, &mut self.rows);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        for op in keys::take_ops(ui) {
            op.apply(&mut self.rows, &mut self.ids, self.count);
        }

        let status = match &self.ticker {
            Some(clock) => spec::ticker_status(self.rows.len(), clock.tick()),
            None => spec::status(self.rows.len()),
        };
        page::header(ui, &status, |_| {});

        let mut clicked = None;
        page::list(ui, |ui| {
            for row in &self.rows {
                if page::row(ui, row, self.selected == Some(row.id)).clicked() {
                    clicked = Some(row.id);
                }
            }
        });
        if let Some(id) = clicked {
            self.toggle_selection(id);
        }
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        page::BACKGROUND.to_normalized_gamma_f32()
    }
}
