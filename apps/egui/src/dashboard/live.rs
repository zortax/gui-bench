//! The live updates: the switch of the top bar, the spinner beside it and the tick that the
//! metric cards show. egui has no timers and no animation API for a turning shape; like its own
//! `Spinner` widget (`egui/src/widgets/spinner.rs`), both take their state from the input time
//! and ask for the repaint they need.

use std::f64::consts::TAU;
use std::time::Duration;

use eframe::egui::{Sense, Ui, epaint::RectShape, vec2};

use super::style::color;
use super::{spec, switch};

/// The live state: whether it is on, and when it was turned on.
#[derive(Default)]
pub struct Live {
    on: bool,
    /// The input time of the click that turned the switch on.
    since: f64,
}

impl Live {
    /// Shows the live switch; a click toggles the live updates.
    pub fn switch(&mut self, ui: &mut Ui) {
        if switch::toggle(ui, &mut self.on).changed() {
            self.since = ui.input(|i| i.time);
        }
    }

    /// Adds the spinner: its room always, and while live is on the square, turned by the time
    /// within the current period.
    pub fn spinner(&self, ui: &mut Ui) {
        let size = vec2(spec::SPINNER_SIZE, spec::SPINNER_SIZE);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        if self.on && ui.is_rect_visible(rect) {
            ui.request_repaint(); // because it is animated
            let period = spec::SPINNER_PERIOD_MS as f64 / 1000.0;
            let turn = ui.input(|i| i.time) % period / period;
            let shape = RectShape::filled(rect, spec::SPINNER_RADIUS, color(spec::PRIMARY))
                .with_angle((turn * TAU) as f32);
            ui.painter().add(shape);
        }
    }

    /// Returns the tick of the metric values: the intervals since live was turned on, 0 while it
    /// is off. Asks for a repaint at the next interval.
    pub fn tick(&self, ui: &Ui) -> u64 {
        if !self.on {
            return 0;
        }
        let interval = Duration::from_millis(spec::LIVE_INTERVAL_MS);
        ui.request_repaint_after(interval);
        let elapsed = ui.input(|i| i.time) - self.since;
        (elapsed / interval.as_secs_f64()) as u64
    }
}
