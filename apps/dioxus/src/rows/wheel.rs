//! A wheel detent moves the list at once, by the axis distance of the detent.
//!
//! Blitz scrolls the container under the pointer by a wheel event's delta at once, with no
//! smoothing and no momentum (a fling only ends a touch-screen pan). Touchpad pixel deltas move
//! the content by exactly the reported pixels, so they keep that default path. Wheel line deltas
//! move 20 px per line in Blitz; this handler takes them over and moves the list by 15 px per
//! line instead, the distance Wayland compositors report for a detent.

use std::rc::Rc;

use dioxus::html::geometry::{PixelsVector2D, WheelDelta};
use dioxus::prelude::*;

/// The axis distance of one wheel detent in logical pixels, as Wayland compositors report it.
const DETENT_PX: f64 = 15.0;

/// The mounted scroll container that the detents move.
#[derive(Clone, Copy)]
pub struct DetentScroll {
    list: Signal<Option<Rc<MountedData>>>,
}

/// Creates the detent handling of a scroll container in the current component.
pub fn use_detent_scroll() -> DetentScroll {
    DetentScroll {
        list: use_signal(|| None),
    }
}

impl DetentScroll {
    /// Keeps the scroll container once it is mounted.
    pub fn mounted(mut self, event: MountedEvent) {
        self.list.set(Some(event.data()));
    }

    /// Moves the container by the detents of a wheel event and stops Blitz's own scroll. Calls
    /// `scrolled` with the new offset, since Blitz sends no scroll event for a scroll that the app
    /// asks for.
    pub fn scroll(self, event: WheelEvent, scrolled: impl FnOnce(PixelsVector2D) + 'static) {
        let WheelDelta::Lines(lines) = event.delta() else {
            return;
        };
        event.prevent_default();
        let Some(list) = self.list.peek().clone() else {
            return;
        };
        spawn(async move {
            let Ok(offset) = list.get_scroll_offset().await else {
                return;
            };
            let target = PixelsVector2D::new(
                offset.x - lines.x * DETENT_PX,
                offset.y - lines.y * DETENT_PX,
            );
            if list.scroll(target, ScrollBehavior::Instant).await.is_ok()
                && let Ok(offset) = list.get_scroll_offset().await
            {
                scrolled(offset);
            }
        });
    }
}
