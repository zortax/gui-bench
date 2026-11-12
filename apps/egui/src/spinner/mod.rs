//! The spinner scenario: a header whose square turns once a second above static rows. egui has
//! no animation API for this; like its own `Spinner` widget (`egui/src/widgets/spinner.rs`), the
//! square takes its angle from the input time and asks for the next repaint while it is visible.

use std::f64::consts::TAU;

use eframe::egui::{self, Sense, Ui, epaint::RectShape, vec2};

use crate::page::{self, color};
use crate::spec::{self, Ids, Row};
use crate::window;

/// Opens the spinner window with `count` rows.
pub fn run(title: &str, count: usize) -> eframe::Result {
    window::run(title, page::setup, move || Box::new(Spinner::new(count)))
}

/// The static rows.
struct Spinner {
    rows: Vec<Row>,
}

impl Spinner {
    fn new(count: usize) -> Self {
        Self {
            rows: Ids::default().rows(count),
        }
    }
}

/// Adds the square, turned by the time within the current period.
fn spinner(ui: &mut Ui) {
    let size = vec2(spec::SPINNER_SIZE, spec::SPINNER_SIZE);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.request_repaint(); // because it is animated
        let period = spec::SPINNER_PERIOD_MS as f64 / 1000.0;
        let turn = ui.input(|i| i.time) % period / period;
        let shape = RectShape::filled(rect, spec::SPINNER_RADIUS, color(spec::SPINNER_COLOR))
            .with_angle((turn * TAU) as f32);
        ui.painter().add(shape);
    }
}

impl eframe::App for Spinner {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        page::header(ui, &spec::status(self.rows.len()), spinner);
        page::list(ui, |ui| {
            for row in &self.rows {
                page::row(ui, row, false);
            }
        });
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        page::BACKGROUND.to_normalized_gamma_f32()
    }
}
