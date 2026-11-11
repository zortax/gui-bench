//! The spinner scenario: a header whose square turns once a second, above static rows.
//!
//! vizia's keyframe animations (`AnimationBuilder`, CSS `@keyframes`) play for a set time and
//! cannot repeat forever, so a timer drives the turn instead: every tick sets a rotation signal
//! that the square binds with `rotate`. The angle comes from the time since the start, so the
//! square turns at exactly one turn per period however the ticks fall.

use std::time::Instant;

use vizia::prelude::*;

use crate::page;
use crate::spec::{self, Ids};

/// The time between two steps of the turn, about one frame at 60 Hz.
const STEP: Duration = Duration::from_millis(16);

/// The time of one turn.
const PERIOD: Duration = Duration::from_millis(spec::SPINNER_PERIOD_MS);

/// Builds the page with `count` static rows and the spinner.
pub fn build(cx: &mut Context, count: usize) {
    let rows = Ids::default().rows(count);
    let angle = Signal::new(Angle::Deg(0.0));
    let start = Instant::now();
    let timer = cx.add_timer(STEP, None, move |_, action| {
        if let TimerAction::Tick(_) = action {
            let turns = start.elapsed().as_secs_f64() / PERIOD.as_secs_f64();
            angle.set(Angle::Deg(360.0 * turns.fract() as f32));
        }
    });
    cx.start_timer(timer);

    page::build(
        cx,
        spec::status(count),
        move |cx| {
            Element::new(cx).class("spinner").rotate(angle);
        },
        move |cx| {
            page::scroll(cx, move |cx| {
                for row in &rows {
                    page::plain_row(cx, row);
                }
            });
        },
    );
}
