//! The live updates: while the switch of the top bar is on, a timer advances the tick that the
//! metric cards show.

use std::time::Duration;

use zgui::prelude::*;
use zgui::view::IntervalHandle;
use zgui::reactive::LocalStorage;

use super::spec;

/// The live state: whether it is on, and the tick of the metric values.
#[derive(Clone, Copy)]
pub struct Live {
    /// Whether the live switch is on.
    pub on: RwSignal<bool, LocalStorage>,
    /// The tick the metric cards show; 0 while live is off.
    pub tick: RwSignal<u64>,
    timer: RwSignal<Option<IntervalHandle>, LocalStorage>,
}

impl Live {
    /// Live off at tick 0.
    pub fn new() -> Self {
        Self {
            on: RwSignal::new_local(false),
            tick: RwSignal::new(0),
            timer: RwSignal::new_local(None),
        }
    }

    /// Starts or stops the timer. `timers` is the clock of the window, which the caller takes in
    /// its component body.
    pub fn run(&self, timers: Option<&Timers>, on: bool) {
        let tick = self.tick;
        let timer = on.then(|| timers).flatten().map(|timers| {
            timers.set_interval(Duration::from_millis(spec::LIVE_INTERVAL_MS), move || {
                tick.update(|tick| *tick += 1);
            })
        });
        self.timer.set(timer);
        if !on {
            tick.set(0);
        }
    }
}
