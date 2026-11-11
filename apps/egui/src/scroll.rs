//! The 1:1 scroll of every list: the app takes the wheel distance out of the raw input (see
//! `wheel`) and moves the scroll area's offset by it in the next pass. The area itself only
//! scrolls by its scroll bar.

use eframe::egui::{
    RawInput, ScrollArea, scroll_area::ScrollAreaOutput, scroll_area::ScrollSource,
};

use crate::wheel;

/// The scroll state of one vertical scroll area.
#[derive(Default)]
pub struct Scroll {
    /// The scroll offset after the last pass.
    offset: f32,
    /// The scroll distance that arrived since the last pass.
    pending: f32,
}

impl Scroll {
    /// Takes the wheel events out of `input`; call it from `App::raw_input_hook`.
    pub fn take(&mut self, input: &mut RawInput) {
        self.pending += wheel::take_scroll(input);
    }

    /// Returns the vertical scroll area that fills the rest of the ui, moved by the pending
    /// distance.
    pub fn area(&mut self) -> ScrollArea {
        let mut area = ScrollArea::vertical()
            .auto_shrink(false)
            .scroll_source(ScrollSource::SCROLL_BAR);
        if self.pending != 0.0 {
            area = area.vertical_scroll_offset(self.offset + self.pending);
            self.pending = 0.0;
        }
        area
    }

    /// Keeps the offset the area ended the pass with.
    pub fn end<R>(&mut self, output: &ScrollAreaOutput<R>) {
        self.offset = output.state.clamped_offset().y;
    }
}
