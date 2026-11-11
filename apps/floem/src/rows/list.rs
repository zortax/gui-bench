//! The list: one vertical stack of every row, built up front, inside a scroll container that
//! fills the window.

use floem::kurbo::Vec2;
use floem::prelude::*;

use super::row::row;
use super::wheel::on_wheel;

const BACKGROUND: Color = Color::from_rgb8(0xff, 0xff, 0xff);

/// Returns the window content: the scroll container with `count` rows.
pub fn view(count: usize) -> impl IntoView {
    let wheel = RwSignal::new(Vec2::ZERO);
    Stack::vertical_from_iter((0..count).map(row))
        .style(|s| s.width_full())
        .on_event(el::PointerWheel, move |_, event| on_wheel(wheel, event))
        .scroll()
        .scroll_delta(move || wheel.get())
        .style(|s| s.size_full().background(BACKGROUND))
}
