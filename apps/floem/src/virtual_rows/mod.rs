//! The virtual scenario: the rows of the rows scenario in floem's `VirtualStack`, which builds
//! only the rows in and near the viewport. The stack takes the index range as its data and a
//! fixed row height, and scrolls like the rows scenario, wheel steps included.

use floem::kurbo::Vec2;
use floem::prelude::*;

use crate::page::color;
use crate::rows::{row, wheel};
use crate::spec;

/// Returns the window content: the virtualized list of `count` rows in a scroll container.
pub fn view(count: usize) -> impl IntoView {
    let delta = RwSignal::new(Vec2::ZERO);
    VirtualStack::with_view(move || 0..count, row::row)
        .item_size_fixed(|| f64::from(spec::ROW_HEIGHT))
        .style(|s| s.flex_col().width_full())
        .on_event(el::PointerWheel, move |_, event| {
            wheel::on_wheel(delta, event)
        })
        .scroll()
        .scroll_delta(move || delta.get())
        .style(|s| s.size_full().background(color(spec::BACKGROUND)))
}
