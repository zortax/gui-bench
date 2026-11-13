//! The live updates: while the switch of the top bar is on, a timer task advances the tick that
//! the metric cards show.

use std::time::Duration;

use gpui_kit::*;

use super::spec;

/// The live state. The views that show it observe it.
#[derive(Default)]
pub struct Live {
    /// Whether the live switch is on.
    pub on: bool,
    /// The tick the metric cards show; 0 while live is off.
    pub tick: u64,
    /// The timer, which stops when it is dropped.
    timer: Option<Task<()>>,
}

impl Live {
    /// Turns the live updates on or off, and goes back to tick 0.
    pub fn set(&mut self, on: bool, cx: &mut Context<Self>) {
        self.on = on;
        self.tick = 0;
        self.timer = on.then(|| {
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(spec::LIVE_INTERVAL_MS))
                        .await;
                    let ticked = this.update(cx, |live, cx| {
                        live.tick += 1;
                        cx.notify();
                    });
                    if ticked.is_err() {
                        break;
                    }
                }
            })
        });
        cx.notify();
    }
}
