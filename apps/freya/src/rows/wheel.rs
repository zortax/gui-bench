//! Wheel input for the rows: moves the scroll view at once by the axis distance in logical
//! pixels. The handler stops the event, so the scroll view never sees it.
//!
//! freya-winit multiplies touchpad pixel deltas and wheel line deltas by fixed factors, and the
//! scroll view animates both and flings at the end of a touchpad gesture. This handler divides
//! the factors out and sets the position with no animation. It keeps the fractional offset, so
//! the integer scroll position follows the summed input.

use freya::prelude::*;

/// freya-winit multiplies touchpad pixel deltas by this factor.
const TOUCHPAD_SPEED_MODIFIER: f64 = 2.0;
/// freya-winit multiplies wheel line deltas by this factor.
const WHEEL_SPEED_MODIFIER: f64 = 53.0;
/// The axis distance of one wheel detent in logical pixels.
const DETENT_PIXELS: f64 = 15.0;

/// The scroll controller of the rows and the exact offset that the input asks for.
#[derive(Clone, Copy)]
pub struct WheelScroll {
    controller: ScrollController,
    offset: State<f64>,
}

impl WheelScroll {
    /// Creates the state in the current component.
    pub fn use_new() -> Self {
        Self {
            controller: use_scroll_controller(ScrollConfig::default),
            offset: use_state(|| 0.0),
        }
    }

    /// The controller that drives the scroll view.
    pub fn controller(&self) -> ScrollController {
        self.controller
    }

    /// Moves the content by the vertical axis distance of a device wheel event.
    pub fn apply(mut self, event: Event<WheelEventData>) {
        let scale_factor = *Platform::get().current_window().scale_factor.peek();
        let distance = match event.source {
            WheelSource::Pixel => event.delta_y / TOUCHPAD_SPEED_MODIFIER / scale_factor,
            WheelSource::Line => event.delta_y / WHEEL_SPEED_MODIFIER * DETENT_PIXELS,
            WheelSource::Custom => return,
        };
        event.stop_propagation();

        let current = f64::from(self.controller.position().y);
        let mut offset = *self.offset.peek();
        if offset.round() != current {
            offset = current;
        }
        offset += distance;
        self.controller.scroll_to_y(offset.round() as i32);
        let applied = f64::from(self.controller.position().y);
        if applied != offset.round() {
            offset = applied;
        }
        *self.offset.write() = offset;
    }
}
