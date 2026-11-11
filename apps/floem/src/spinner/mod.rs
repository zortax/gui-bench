//! The spinner scenario: a header whose spinner turns once a second, driven frame by frame,
//! above static rows.
//!
//! floem's keyframe animation (`.animation(...)` with `.repeat(true)`) stops for good at this
//! revision: when a frame of the animation leaves the style as it was, nothing is painted, and
//! the style update it schedules for the next frame never runs, since only a paint or another
//! event wakes the update loop. So the spinner uses floem's per-frame callback instead:
//! `exec_after_animation_frame`, re-armed from an effect on the angle signal (the timer
//! example's pattern), sets the angle each frame, and the style reads it.
//!
//! The vger renderer fills a rectangle or a rounded rectangle as the axis-aligned box between its
//! two transformed corners, so a rotated square with a background comes out as a box that changes
//! size instead of turning. The spinner therefore paints its square in a canvas as a path, which
//! vger transforms point by point.

use std::time::{Duration, Instant};

use floem::action::{exec_after, exec_after_animation_frame};
use floem::kurbo::{Rect, Shape};
use floem::prelude::*;
use floem::reactive::Effect;

use crate::page::{self, color, header, plain_rows};
use crate::spec::{self, Ids};

/// The path tolerance of the square's rounded corners.
const TOLERANCE: f64 = 0.1;

/// Returns the page with `count` static rows and the spinner.
pub fn view(count: usize) -> impl IntoView {
    let rows = plain_rows(Ids::default().rows(count));
    page::page(header(move || spec::status(count), spinner()), rows)
}

/// Returns the square that turns one full turn per period, linearly and forever.
fn spinner() -> impl IntoView {
    let side = f64::from(spec::SPINNER_SIZE);
    let period = Duration::from_millis(spec::SPINNER_PERIOD_MS);
    let start = Instant::now();
    let degrees = RwSignal::new(0.0_f64);
    Effect::new(move |_| {
        degrees.track();
        exec_after_animation_frame(move |_| {
            let turns = start.elapsed().as_secs_f64() / period.as_secs_f64();
            degrees.set(turns.fract() * 360.0);
        });
    });
    // The window does not exist yet while its views are built, and a frame callback needs one,
    // so a zero timer, which runs in the event loop, arms the first frame.
    exec_after(Duration::ZERO, move |_| degrees.set(0.0));
    canvas(|cx, size| {
        let square = Rect::ZERO
            .with_size(size)
            .to_rounded_rect(f64::from(spec::SPINNER_RADIUS))
            .to_path(TOLERANCE);
        cx.fill(&square, color(spec::SPINNER_COLOR), 0.0);
    })
    .style(move |s| {
        s.flex_shrink(0.0_f32)
            .size(side, side)
            .rotate(degrees.get().deg())
    })
}
