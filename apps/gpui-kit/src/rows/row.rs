//! One row of the rows and virtual scenarios: zebra background, hover color, label and value.

use gpui_kit::*;

use crate::spec;

/// Returns row `i`.
pub fn row(i: usize) -> Stateful<Div> {
    div()
        .id(i)
        .flex()
        .flex_row()
        .flex_none()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(spec::ROW_HEIGHT))
        .pl(px(spec::PADDING_X))
        .pr(px(spec::PADDING_X))
        .border_b_1()
        .border_color(rgb(spec::ROW_BORDER))
        .bg(rgb(spec::row_background(i)))
        .hover(|style| style.bg(rgb(spec::ROW_HOVER)))
        .text_size(px(spec::FONT_SIZE))
        .child(
            div()
                .text_color(rgb(spec::LABEL_COLOR))
                .child(spec::row_label(i)),
        )
        .child(
            div()
                .text_color(rgb(spec::VALUE_COLOR))
                .child(spec::value(i as u64).to_string()),
        )
}
