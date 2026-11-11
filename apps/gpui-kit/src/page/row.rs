//! A row of the header scenarios: its label on the left and its value on the right.

use gpui_kit::component::h_flex;
use gpui_kit::*;

use crate::spec;

/// Returns a row with `label` and `value`, in the selected color when `selected`.
pub fn plain_row(label: impl Into<SharedString>, value: u32, selected: bool) -> Div {
    let background = if selected {
        spec::SELECTED
    } else {
        spec::ROW_EVEN
    };
    h_flex()
        .flex_none()
        .w_full()
        .h(px(spec::ROW_HEIGHT))
        .px(px(spec::PADDING_X))
        .justify_between()
        .border_b_1()
        .border_color(rgb(spec::ROW_BORDER))
        .bg(rgb(background))
        .child(div().text_color(rgb(spec::LABEL_COLOR)).child(label.into()))
        .child(
            div()
                .text_color(rgb(spec::VALUE_COLOR))
                .child(value.to_string()),
        )
}
