//! The ticker timer, the way floem's timer example drives one: an effect on the tick signal arms
//! a one-shot `exec_after`, whose action bumps the tick, which runs the effect again.

use std::time::Duration;

use floem::action::exec_after;
use floem::prelude::*;
use floem::reactive::Effect;

/// Every `interval`, runs `on_tick` with the next tick number and then sets `tick` to it.
pub fn every(interval: Duration, tick: RwSignal<u64>, on_tick: impl Fn(u64) + Copy + 'static) {
    Effect::new(move |_| {
        tick.track();
        exec_after(interval, move |_| {
            let now = tick.get_untracked() + 1;
            on_tick(now);
            tick.set(now);
        });
    });
}
