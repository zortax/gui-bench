//! The live updates: while the switch of the top bar is on, a floem timer advances the tick that
//! the metric cards show, and the spinner turns.

use std::time::{Duration, Instant};

use floem::action::{TimerToken, exec_after, exec_after_animation_frame};
use floem::kurbo::{Rect, Shape};
use floem::prelude::*;
use floem::reactive::Effect;

use super::style::{color, px};
use crate::dash;

/// The path tolerance of the spinner's rounded corners.
const TOLERANCE: f64 = 0.1;

/// The live state: whether it is on, and the tick of the metric values.
#[derive(Clone, Copy)]
pub struct Live {
    /// Whether the live switch is on.
    pub on: RwSignal<bool>,
    /// The tick the metric cards show; 0 while live is off.
    pub tick: RwSignal<u64>,
}

impl Live {
    /// Live off at tick 0. An effect on the switch and the tick arms the timer of the next tick
    /// while live is on (the pattern of floem's timer example), and brings the tick back to 0
    /// when it goes off.
    pub fn new() -> Self {
        let on = RwSignal::new(false);
        let tick = RwSignal::new(0_u64);
        Effect::new(move |timer: Option<Option<TimerToken>>| {
            if let Some(timer) = timer.flatten() {
                timer.cancel();
            }
            if !on.get() {
                tick.set(0);
                return None;
            }
            tick.track();
            let interval = Duration::from_millis(dash::LIVE_INTERVAL_MS);
            Some(exec_after(interval, move |_| tick.update(|tick| *tick += 1)))
        });
        Self { on, tick }
    }
}

/// Returns the spinner in the space it keeps beside the live switch. The square exists only
/// while live is on.
pub fn spinner(live: Live) -> impl IntoView {
    let side = px(dash::SPINNER_SIZE);
    dyn_container(
        move || live.on.get(),
        |on| {
            if on {
                square().into_any()
            } else {
                Empty::new().into_any()
            }
        },
    )
    .style(move |s| s.flex_shrink(0.0_f32).size(side, side))
}

/// The square that turns one full turn per period, linearly and forever.
///
/// floem's keyframe animation with `.repeat(true)` stops after one pass at this revision, so the
/// square uses floem's per-frame callback instead: `exec_after_animation_frame`, re-armed from an
/// effect on the angle signal (the pattern of the timer example), sets the angle each frame, and
/// the style reads it. The vger renderer fills a rotated rounded rectangle as the axis-aligned
/// box between its transformed corners, so the square is painted in a canvas as a path, which
/// vger transforms point by point.
fn square() -> impl IntoView {
    let side = px(dash::SPINNER_SIZE);
    let period = Duration::from_millis(dash::SPINNER_PERIOD_MS);
    let start = Instant::now();
    let degrees = RwSignal::new(0.0_f64);
    Effect::new(move |_| {
        degrees.track();
        exec_after_animation_frame(move |_| {
            let turns = start.elapsed().as_secs_f64() / period.as_secs_f64();
            degrees.set(turns.fract() * 360.0);
        });
    });
    canvas(|cx, size| {
        let square = Rect::ZERO
            .with_size(size)
            .to_rounded_rect(px(dash::SPINNER_RADIUS))
            .to_path(TOLERANCE);
        cx.fill(&square, color(dash::PRIMARY), 0.0);
    })
    .style(move |s| s.size(side, side).rotate(degrees.get().deg()))
}
