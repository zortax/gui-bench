//! The virtual scenario: the rows of the rows scenario in GPUI's `uniform_list`, which builds
//! only the visible rows and scrolls as it does by default.

use gpui_kit::*;

use crate::rows::row;
use crate::spec;

/// The virtualized list of `count` rows.
pub struct VirtualRows {
    count: usize,
}

impl VirtualRows {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Render for VirtualRows {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        uniform_list("rows", self.count, |range: std::ops::Range<usize>, _, _| {
            range.map(row).collect::<Vec<_>>()
        })
        .size_full()
        .bg(rgb(spec::BACKGROUND))
    }
}
