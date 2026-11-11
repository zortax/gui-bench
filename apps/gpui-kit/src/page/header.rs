//! The header: the status text on the left, and whatever the caller adds on the right.

use gpui_kit::component::h_flex;
use gpui_kit::*;

use crate::spec;

/// Returns a 40 px header that shows `status`. A child added to it sits at the right end.
pub fn header(status: impl Into<SharedString>) -> Div {
    h_flex()
        .flex_none()
        .w_full()
        .h(px(spec::HEADER_HEIGHT))
        .px(px(spec::PADDING_X))
        .justify_between()
        .border_b_1()
        .border_color(rgb(spec::HEADER_BORDER))
        .bg(rgb(spec::HEADER_BG))
        .child(
            div()
                .flex_none()
                .text_color(rgb(spec::LABEL_COLOR))
                .child(status.into()),
        )
}
