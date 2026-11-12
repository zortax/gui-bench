//! The search and typing scenarios: a header with a focused single-line `TextEdit` above the
//! rows. With `filter`, every pass shows the rows whose label holds the text (immediate mode:
//! the filter runs as the rows are laid out); without, all rows stay.

use eframe::egui::{self, TextEdit, Ui};

use crate::page;
use crate::spec::{self, Ids, Row};
use crate::window;

/// Opens the search window with `count` rows; `filter` picks search over typing.
pub fn run(title: &str, count: usize, filter: bool) -> eframe::Result {
    window::run(title, page::setup, move || {
        Box::new(Search::new(count, filter))
    })
}

/// The rows and the text of the input.
struct Search {
    rows: Vec<Row>,
    filter: bool,
    query: String,
    /// Whether the input asked for the focus yet: it does so in the first pass.
    focused: bool,
}

impl Search {
    fn new(count: usize, filter: bool) -> Self {
        Self {
            rows: Ids::default().rows(count),
            filter,
            query: String::new(),
            focused: false,
        }
    }
}

impl eframe::App for Search {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        let status = spec::status(self.rows.len());
        page::header(ui, &status, |ui| {
            let input = ui.add(
                TextEdit::singleline(&mut self.query)
                    .hint_text("Search")
                    .desired_width(spec::INPUT_WIDTH),
            );
            if !self.focused {
                input.request_focus();
                self.focused = true;
            }
        });
        let (filter, query) = (self.filter, &self.query);
        page::list(ui, |ui| {
            for row in &self.rows {
                if !filter || spec::matches(&row.label, query) {
                    page::row(ui, row, false);
                }
            }
        });
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        page::BACKGROUND.to_normalized_gamma_f32()
    }
}
