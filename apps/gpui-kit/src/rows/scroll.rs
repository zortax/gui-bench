//! Scrolling 1:1 with the input: a scroll event moves the content by its axis distance in
//! logical pixels, without the factors that the platform applies.

use gpui_kit::*;

/// The Wayland backend multiplies touchpad axis distances by this factor.
const PLATFORM_PIXEL_SCALE: f32 = 3.0;
/// The Wayland backend reports one wheel detent as this many lines.
const LINES_PER_DETENT: f32 = 3.0;
/// The axis distance of one wheel detent in logical pixels.
const DETENT_PIXELS: f32 = 15.0;

/// Moves the content of `scroll` by the axis distance of the event, within its range. Returns
/// true when the offset changed.
pub fn scroll_by(scroll: &ScrollHandle, event: &ScrollEvent) -> bool {
    let delta = match event.delta {
        ScrollDelta::Pixels(delta) => delta.y / PLATFORM_PIXEL_SCALE,
        ScrollDelta::Lines(lines) => px(lines.y / LINES_PER_DETENT * DETENT_PIXELS),
    };
    let mut offset = scroll.offset();
    let max = scroll.max_offset();
    let y = (offset.y + delta).clamp(-max.y, px(0.0));
    if y == offset.y {
        return false;
    }
    offset.y = y;
    scroll.set_offset(offset);
    true
}
