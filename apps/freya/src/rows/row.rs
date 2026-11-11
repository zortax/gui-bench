//! One row: a 32 px strip with a label on the left and a value on the right. The row holds its
//! own hover state, so a hover change renders only that row.

use freya::prelude::*;

const ROW_HEIGHT: f32 = 32.0;
const PADDING_X: f32 = 12.0;

const ROW_EVEN: Color = Color::from_rgb(0xff, 0xff, 0xff);
const ROW_ODD: Color = Color::from_rgb(0xf6, 0xf8, 0xfa);
const ROW_HOVER: Color = Color::from_rgb(0xcf, 0xe3, 0xff);
const ROW_BORDER: Color = Color::from_rgb(0xe5, 0xe7, 0xeb);
const LABEL_COLOR: Color = Color::from_rgb(0x1f, 0x23, 0x28);
const VALUE_COLOR: Color = Color::from_rgb(0x6e, 0x77, 0x81);

/// The row at an index of the list.
#[derive(PartialEq)]
pub struct Row {
    index: usize,
}

impl Row {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

impl Component for Row {
    fn render(&self) -> impl IntoElement {
        let mut hovered = use_state(|| false);
        let i = self.index;
        let background = if *hovered.read() {
            ROW_HOVER
        } else if i.is_multiple_of(2) {
            ROW_EVEN
        } else {
            ROW_ODD
        };
        let border = Border::new()
            .fill(ROW_BORDER)
            .width(BorderWidth {
                top: 0.0,
                right: 0.0,
                bottom: 1.0,
                left: 0.0,
            })
            .alignment(BorderAlignment::Inner);

        rect()
            .horizontal()
            .width(Size::fill())
            .height(Size::px(ROW_HEIGHT))
            .padding(Gaps::new(0.0, PADDING_X, 0.0, PADDING_X))
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
            .background(background)
            .border(border)
            .on_pointer_enter(move |_| hovered.set_if_modified(true))
            .on_pointer_leave(move |_| hovered.set_if_modified(false))
            .child(label().color(LABEL_COLOR).text(format!("Row {i:05}")))
            .child(
                label()
                    .color(VALUE_COLOR)
                    .text(format!("{}", (i * 7919) % 10000)),
            )
    }
}
