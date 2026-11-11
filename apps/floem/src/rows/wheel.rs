//! Wheel steps: floem turns one line of a line delta into 20 px. The handler here takes line
//! deltas off the rows and hands the scroll container 15 px per detent instead. Pixel deltas
//! pass on to the scroll container, which applies them 1:1 in logical pixels.

use floem::event::EventPropagation;
use floem::kurbo::Vec2;
use floem::prelude::*;
use floem::ui_events::ScrollDelta;
use floem::ui_events::pointer::PointerScrollEvent;

/// The axis distance of one wheel detent in logical pixels.
const DETENT_PIXELS: f64 = 15.0;

/// Returns the scroll offset change of a line delta, or `None` for other deltas. winit reports
/// one detent as one line, positive towards the top of the content.
pub fn line_offset(event: &PointerScrollEvent) -> Option<Vec2> {
    match event.delta {
        ScrollDelta::LineDelta(x, y) => Some(Vec2::new(
            -f64::from(x) * DETENT_PIXELS,
            -f64::from(y) * DETENT_PIXELS,
        )),
        _ => None,
    }
}

/// Stops a line delta and stores its offset change in `delta`, which feeds the scroll container.
pub fn on_wheel(delta: RwSignal<Vec2>, event: &PointerScrollEvent) -> EventPropagation {
    match line_offset(event) {
        Some(offset) => {
            delta.set(offset);
            EventPropagation::Stop
        }
        None => EventPropagation::Continue,
    }
}
