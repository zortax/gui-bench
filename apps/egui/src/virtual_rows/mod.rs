//! The virtual scenario: the rows of the rows scenario in `ScrollArea::show_rows`, which lays
//! out only the rows in the viewport.

use eframe::egui::{self, ScrollArea, Ui};

use crate::rows::{self, row};
use crate::{spec, window};

/// Opens the virtual window with `count` rows.
pub fn run(title: &str, count: usize) -> eframe::Result {
    window::run(title, rows::setup, move || {
        Box::new(VirtualRows::new(count))
    })
}

/// The row count.
struct VirtualRows {
    count: usize,
}

impl VirtualRows {
    fn new(count: usize) -> Self {
        Self { count }
    }
}

impl eframe::App for VirtualRows {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        ScrollArea::vertical().auto_shrink(false).show_rows(
            ui,
            spec::ROW_HEIGHT,
            self.count,
            |ui, range| {
                for i in range {
                    row::row(ui, i);
                }
            },
        );
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        egui::Color32::WHITE.to_normalized_gamma_f32()
    }
}
