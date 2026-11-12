//! The list: one vertical stack of every row, built up front, inside a scroll container that
//! fills the window.

use floem::prelude::*;

use super::row::row;

const BACKGROUND: Color = Color::from_rgb8(0xff, 0xff, 0xff);

/// Returns the window content: the scroll container with `count` rows.
pub fn view(count: usize) -> impl IntoView {
    Stack::vertical_from_iter((0..count).map(row))
        .style(|s| s.width_full())
        .scroll()
        .style(|s| s.size_full().background(BACKGROUND))
}
