//! The ticker clock. egui has no timers: the app asks for a repaint when the next tick is due
//! (`Context::request_repaint_after`) and counts the ticks from the input time, one tick per
//! interval that has passed since the first pass.

use std::time::Duration;

use eframe::egui::Context;

use crate::cli::Ticker;
use crate::spec::{self, Row};

/// The ticks of a ticker and the time they count from.
pub struct Clock {
    ticker: Ticker,
    /// The input time of the first pass, in seconds.
    start: Option<f64>,
    tick: u64,
}

impl Clock {
    pub fn new(ticker: Ticker) -> Self {
        Self {
            ticker,
            start: None,
            tick: 0,
        }
    }

    /// Returns the ticks so far.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Runs every tick that is due: each one counts up and updates the ticker rows. Then asks for
    /// a repaint at the next tick.
    pub fn advance(&mut self, ctx: &Context, rows: &mut [Row]) {
        let now = ctx.input(|i| i.time);
        let start = *self.start.get_or_insert(now);
        let interval = Duration::from_millis(self.ticker.interval_ms.max(1)).as_secs_f64();
        let due = ((now - start) / interval) as u64;
        while self.tick < due {
            self.tick += 1;
            for at in spec::ticker_positions(rows.len(), self.ticker.updates) {
                let row = &mut rows[at];
                row.value = spec::ticker_value(row.id, self.tick);
            }
        }
        let next = start + (self.tick + 1) as f64 * interval;
        ctx.request_repaint_after(Duration::from_secs_f64((next - now).max(0.0)));
    }
}
