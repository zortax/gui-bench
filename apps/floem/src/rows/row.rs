//! One row: a 32 px band with the row label on the left and its value on the right. The band
//! takes the hover color at once while the pointer is over it.

use floem::prelude::*;

/// The height of one row in logical pixels, bottom border included.
const ROW_HEIGHT: f64 = 32.0;
const PADDING_X: f64 = 12.0;
const FONT_SIZE: f64 = 14.0;
const FONT_FAMILY: &str = "sans-serif";

const ROW_EVEN: Color = Color::from_rgb8(0xff, 0xff, 0xff);
const ROW_ODD: Color = Color::from_rgb8(0xf6, 0xf8, 0xfa);
const ROW_HOVER: Color = Color::from_rgb8(0xcf, 0xe3, 0xff);
const ROW_BORDER: Color = Color::from_rgb8(0xe5, 0xe7, 0xeb);
const LABEL_COLOR: Color = Color::from_rgb8(0x1f, 0x23, 0x28);
const VALUE_COLOR: Color = Color::from_rgb8(0x6e, 0x77, 0x81);

/// Returns the row with index `i`.
pub fn row(i: usize) -> impl IntoView {
    let background = if i.is_multiple_of(2) {
        ROW_EVEN
    } else {
        ROW_ODD
    };
    let label = text(format!("Row {i:05}"), LABEL_COLOR);
    let value = text(format!("{}", (i * 7919) % 10000), VALUE_COLOR);
    Stack::horizontal((label, value)).style(move |s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(ROW_HEIGHT)
            .padding_horiz(PADDING_X)
            .items_center()
            .justify_between()
            .border_bottom(1.0)
            .border_color(ROW_BORDER)
            .background(background)
            .hover(|s| s.background(ROW_HOVER))
    })
}

/// Returns a static text in the row font.
fn text(content: String, color: Color) -> impl IntoView {
    Label::new(content).style(move |s| {
        s.font_family(FONT_FAMILY.to_owned())
            .font_size(FONT_SIZE)
            .color(color)
            .selectable(false)
    })
}
