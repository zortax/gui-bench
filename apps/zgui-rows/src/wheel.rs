//! A wheel detent moves the content at once, by the axis distance of the detent.
//!
//! Touchpad and other pixel deltas keep the framework's own handling, which moves the content by
//! exactly the reported pixels and adds no momentum on Wayland.

use zgui::geom::{DevicePx, Point};
use zgui::prelude::*;
use zgui::vocab::{ScrollPhase, WheelEvent};

/// The axis distance of one wheel detent in logical pixels, as Wayland compositors report it.
const DETENT_PX: f32 = 15.0;

/// Scrolls `list` by the detents of a wheel event, with no glide.
///
/// Returns true when it took the scroll, so the caller prevents the default glide.
pub fn scroll_detents(ev: &WheelEvent, list: &NodeRef) -> bool {
    let ScrollDelta::Lines { x, y } = ev.delta else {
        return false;
    };
    if ev.phase != ScrollPhase::Discrete {
        return false;
    }
    let scale = list.scale();
    list.scroll_to(
        ScrollTarget::By(Point::new(
            DevicePx(x * DETENT_PX * scale),
            DevicePx(y * DETENT_PX * scale),
        )),
        ScrollBehavior::Instant,
    );
    true
}
