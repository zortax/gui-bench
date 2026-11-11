//! The rows view: a full-window vertical scroll container that holds every row as a real
//! element, and scrolls 1:1 with the input in logical pixels.

mod row;
pub mod scroll;

use gpui_kit::*;

pub use self::row::row;
use crate::spec;

pub struct Rows {
    count: usize,
    scroll: ScrollHandle,
}

impl Rows {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            scroll: ScrollHandle::new(),
        }
    }

    /// Moves the content by the axis distance of the event and stops the built-in scroll
    /// handler, which applies the platform line and pixel factors.
    fn on_scroll(&mut self, event: &ScrollEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        if scroll::scroll_by(&self.scroll, event) {
            cx.notify();
        }
    }
}

impl Render for Rows {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("scroll")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .bg(rgb(spec::BACKGROUND))
            .child(
                div()
                    .id("rows")
                    .flex()
                    .flex_col()
                    .w_full()
                    .on_scroll(cx.listener(Self::on_scroll))
                    .children((0..self.count).map(row)),
            )
    }
}
