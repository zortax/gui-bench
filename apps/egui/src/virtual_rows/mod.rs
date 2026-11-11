//! The virtual scenario: the rows of the rows scenario in `ScrollArea::show_rows`, which lays
//! out only the rows in the viewport. It scrolls 1:1 like the rows scenario.

use eframe::egui::{self, Context, RawInput, Ui};

use crate::rows::{self, row};
use crate::scroll::Scroll;
use crate::{spec, window};

/// Opens the virtual window with `count` rows.
pub fn run(title: &str, count: usize) -> eframe::Result {
    window::run(title, rows::setup, move || {
        Box::new(VirtualRows::new(count))
    })
}

/// The row count and the scroll state.
struct VirtualRows {
    count: usize,
    scroll: Scroll,
}

impl VirtualRows {
    fn new(count: usize) -> Self {
        Self {
            count,
            scroll: Scroll::default(),
        }
    }
}

impl eframe::App for VirtualRows {
    fn raw_input_hook(&mut self, _: &Context, input: &mut RawInput) {
        self.scroll.take(input);
    }

    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        let output = self
            .scroll
            .area()
            .show_rows(ui, spec::ROW_HEIGHT, self.count, |ui, range| {
                for i in range {
                    row::row(ui, i);
                }
            });
        self.scroll.end(&output);
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        egui::Color32::WHITE.to_normalized_gamma_f32()
    }
}
