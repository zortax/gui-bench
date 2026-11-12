//! The rows view: a full-window vertical scroll container that holds every row as a real
//! element. It scrolls as GPUI's `overflow_y_scroll` does by default.

mod row;

use gpui_kit::*;

pub use self::row::row;
use crate::spec;

pub struct Rows {
    count: usize,
}

impl Rows {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl Render for Rows {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("scroll")
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(spec::BACKGROUND))
            .child(
                div()
                    .id("rows")
                    .flex()
                    .flex_col()
                    .w_full()
                    .children((0..self.count).map(row)),
            )
    }
}
