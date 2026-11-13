//! The list scenario: the header of the orders in a top panel, and under it the order rows in
//! `ScrollArea::show_rows`, which lays out only the rows in the viewport.

use eframe::egui::{self, CentralPanel, Frame, Panel, ScrollArea, Ui};

use super::{orders, spec, style};
use crate::window;

/// The space between the amount column and the right edge of the window.
const AMOUNT_INSET: f32 = 24.0;

/// Opens the list window with `count` orders and runs it until it closes.
pub fn run(title: &str, count: usize) -> eframe::Result {
    window::run(title, style::setup, move || Box::new(List::new(count)))
}

/// The orders of the list.
struct List {
    count: usize,
    /// Whether the orders are sorted by amount.
    sorted: bool,
    /// The index of the order of each row, made again when the sorting changes.
    order: Vec<u32>,
}

impl List {
    fn new(count: usize) -> Self {
        Self {
            count,
            sorted: false,
            order: spec::list_order(count, false),
        }
    }
}

impl eframe::App for List {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        Panel::top("header")
            .exact_size(spec::TABLE_HEADER_HEIGHT)
            .show_separator_line(false)
            .frame(Frame::new())
            .show(ui, |ui| {
                let end = ui.available_width() - AMOUNT_INSET;
                if orders::header(ui, 0, end, self.sorted) {
                    self.sorted = !self.sorted;
                    self.order = spec::list_order(self.count, self.sorted);
                }
            });
        let rows = Frame::new().fill(style::color(spec::CARD_BG));
        CentralPanel::default().frame(rows).show(ui, |ui| {
            orders::rows_style(ui);
            let end = ui.available_width() - AMOUNT_INSET;
            ScrollArea::vertical().auto_shrink(false).show_rows(
                ui,
                spec::ORDER_ROW_HEIGHT,
                self.count,
                |ui, range| {
                    for row in range {
                        let order = spec::order(self.order[row] as usize);
                        orders::row(ui, &order, end, false);
                    }
                },
            );
        });
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        style::color(spec::CARD_BG).to_normalized_gamma_f32()
    }
}
