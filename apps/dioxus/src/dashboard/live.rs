//! The live updates: while the switch of the top bar is on, a timer advances the tick that the
//! metric cards show.

use std::time::Duration;

use dioxus::prelude::*;

use super::spec;

/// The live state: whether it is on, and the tick of the metric values.
#[derive(Clone, Copy, PartialEq)]
pub struct Live {
    /// Whether the live switch is on.
    pub on: Signal<bool>,
    /// The tick the metric cards show; 0 while live is off.
    pub tick: Signal<u64>,
}

/// Returns the live state, off at tick 0. A resource follows the switch: while it is on, its
/// future waits the interval with tokio's sleep and counts the tick up; turning the switch off
/// cancels that future and sets the tick back to 0.
pub fn use_live() -> Live {
    let on = use_signal(|| false);
    let mut tick = use_signal(|| 0_u64);
    use_resource(move || async move {
        if !on() {
            tick.set(0);
            return;
        }
        loop {
            tokio::time::sleep(Duration::from_millis(spec::LIVE_INTERVAL_MS)).await;
            tick += 1;
        }
    });
    Live { on, tick }
}
