//! The live updates: while the switch of the top bar is on, one timer advances the tick that the
//! metric cards show and another turns the spinner.
//!
//! vizia's keyframe animations (`AnimationBuilder`, CSS `@keyframes`) play once for a set time and
//! cannot repeat, so a timer drives the turn: every step sets the rotation signal that the
//! spinner binds with `rotate`. The angle comes from the time since the start, so the spinner
//! turns once per period however the steps fall.

use std::time::Instant;

use vizia::prelude::*;

use crate::dashboard_spec::{LIVE_INTERVAL_MS, SPINNER_PERIOD_MS};

/// The time between two steps of the turn, about one frame at 60 Hz.
const STEP: Duration = Duration::from_millis(16);

/// The live state: whether it is on, the tick of the metric values and the angle of the spinner.
#[derive(Clone, Copy)]
pub struct Live {
    /// Whether the live switch is on.
    pub on: Signal<bool>,
    /// The tick the metric cards show; 0 while live is off.
    pub tick: Signal<u64>,
    /// The rotation of the spinner.
    pub angle: Signal<Angle>,
    ticker: Timer,
    turner: Timer,
}

impl Live {
    /// Live off at tick 0, with its two timers added but not started.
    pub fn new(cx: &mut Context) -> Self {
        let tick = Signal::new(0_u64);
        let angle = Signal::new(Angle::Deg(0.0));
        let ticker = cx.add_timer(
            Duration::from_millis(LIVE_INTERVAL_MS),
            None,
            move |_, action| {
                if let TimerAction::Tick(_) = action {
                    tick.update(|tick| *tick += 1);
                }
            },
        );
        let start = Instant::now();
        let turner = cx.add_timer(STEP, None, move |_, action| {
            if let TimerAction::Tick(_) = action {
                let turns = start.elapsed().as_millis() as f64 / SPINNER_PERIOD_MS as f64;
                angle.set(Angle::Deg(360.0 * turns.fract() as f32));
            }
        });
        Self {
            on: Signal::new(false),
            tick,
            angle,
            ticker,
            turner,
        }
    }

    /// Turns live on or off: starts or stops the timers, and shows the startup values when off.
    pub fn toggle(&self, cx: &mut EventContext) {
        let on = !self.on.get();
        self.on.set(on);
        if on {
            cx.start_timer(self.ticker);
            cx.start_timer(self.turner);
        } else {
            cx.stop_timer(self.ticker);
            cx.stop_timer(self.turner);
            self.tick.set(0);
        }
    }
}
