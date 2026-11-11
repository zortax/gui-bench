//! Scroll input: takes every wheel event out of the raw input and sums its axis distance in
//! logical pixels, so a list moves its scroll offset by that sum in the same frame (see `scroll`).
//!
//! egui smooths wheel events over several frames, ignores the distance of an event with the start
//! phase, and clears the distance it has not applied at the end phase
//! (`egui/src/input_state/wheel_state.rs`). winit on Wayland gives the first finger axis event of
//! a gesture the start phase (`winit/src/platform_impl/linux/wayland/seat/pointer/mod.rs`).

use eframe::egui::{Event, MouseWheelUnit, RawInput};

/// The axis distance of one wheel detent in logical pixels, as Wayland compositors report it.
const DETENT_PX: f32 = 15.0;

/// Removes the wheel events from `input` and returns the change of the scroll offset in logical
/// pixels. A positive value moves the view down the list.
///
/// Point deltas are logical pixels at scale 1: winit multiplies the Wayland axis value by the
/// scale factor and egui-winit divides it by the same factor (`egui-winit/src/lib.rs`,
/// `on_mouse_wheel`).
pub fn take_scroll(input: &mut RawInput) -> f32 {
    let viewport_height = input.screen_rect.map_or(0.0, |rect| rect.height());
    let mut offset = 0.0;
    input.events.retain(|event| {
        let Event::MouseWheel { unit, delta, .. } = event else {
            return true;
        };
        let pixels = match unit {
            MouseWheelUnit::Point => delta.y,
            MouseWheelUnit::Line => delta.y * DETENT_PX,
            MouseWheelUnit::Page => delta.y * viewport_height,
        };
        // A wheel delta moves the content, and the offset moves the view.
        offset -= pixels;
        false
    });
    offset
}
