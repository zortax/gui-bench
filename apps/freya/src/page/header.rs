//! The header: the status text on the left and a slot on the right.

use freya::prelude::*;

use super::{bottom_border, color};
use crate::spec;

/// Returns the 40 px header that shows `status` on the left and `end` on the right. The border
/// takes no layout space in freya, so the bottom padding keeps the items centered above it.
pub fn header(status: String, end: impl IntoElement) -> Rect {
    rect()
        .horizontal()
        .width(Size::fill())
        .height(Size::px(spec::HEADER_HEIGHT))
        .padding(Gaps::new(
            0.0,
            spec::PADDING_X,
            spec::BORDER_HEIGHT,
            spec::PADDING_X,
        ))
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .background(color(spec::HEADER_BG))
        .border(bottom_border(spec::HEADER_BORDER))
        .child(label().color(color(spec::LABEL_COLOR)).text(status))
        .child(end)
}
