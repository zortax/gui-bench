//! The virtual scenario: the rows of the rows scenario in GPUI's `uniform_list`, which builds
//! only the visible rows. Each built row takes the scroll events first and moves the list 1:1
//! with the input, as the rows scenario does, before the list's own handler scales them.

use gpui_kit::*;

use crate::rows::{row, scroll};
use crate::spec;

/// The virtualized list of `count` rows.
pub struct VirtualRows {
    count: usize,
    scroll: UniformListScrollHandle,
}

impl VirtualRows {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            scroll: UniformListScrollHandle::new(),
        }
    }

    /// Moves the list by the axis distance of the event and stops the list's own handler.
    fn on_scroll(&mut self, event: &ScrollEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        let handle = self.scroll.0.borrow().base_handle.clone();
        if scroll::scroll_by(&handle, event) {
            cx.notify();
        }
    }
}

impl Render for VirtualRows {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        uniform_list(
            "rows",
            self.count,
            cx.processor(|_, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|i| row(i).on_scroll(cx.listener(Self::on_scroll)))
                    .collect::<Vec<_>>()
            }),
        )
        .track_scroll(&self.scroll)
        .size_full()
        .bg(rgb(spec::BACKGROUND))
    }
}
