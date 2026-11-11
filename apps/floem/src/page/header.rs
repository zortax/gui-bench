//! The header: the status text on the left and a slot on the right.

use floem::prelude::*;

use super::color;
use crate::spec;

/// Returns the 40 px header that shows `status` and `slot`.
pub fn header<S: std::fmt::Display + 'static>(
    status: impl Fn() -> S + 'static,
    slot: impl IntoView + 'static,
) -> Stack {
    let status = Label::derived(status).style(|s| s.color(color(spec::LABEL_COLOR)));
    Stack::horizontal((status, slot)).style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(f64::from(spec::HEADER_HEIGHT))
            .padding_horiz(f64::from(spec::PADDING_X))
            .items_center()
            .justify_between()
            .border_bottom(f64::from(spec::BORDER_HEIGHT))
            .border_color(color(spec::HEADER_BORDER))
            .background(color(spec::HEADER_BG))
    })
}
