//! The page of the header scenarios: a header with the status text and a slot on the right, and
//! a scroll container of rows below it. Rows here have one background and no hover color.

mod header;
mod row;

use floem::prelude::*;

pub use header::header;
pub use row::{band, plain_row, plain_rows, selected_style};

use crate::spec;

/// Returns a spec color.
pub const fn color(hex: u32) -> Color {
    let (r, g, b) = spec::rgb(hex);
    Color::from_rgb8(r, g, b)
}

/// Returns the page: `header` on top and `rows` in a scroll container that fills the rest. The
/// page sets the font that every text inherits.
pub fn page(header: impl IntoView + 'static, rows: impl IntoView + 'static) -> Stack {
    let list = rows.scroll().style(|s| {
        s.flex_grow(1.0_f32)
            .min_height(0.0)
            .width_full()
            .background(color(spec::BACKGROUND))
    });
    Stack::vertical((header, list)).style(|s| {
        s.size_full()
            .background(color(spec::BACKGROUND))
            .font_family(spec::FONT_FAMILY.to_owned())
            .font_size(f64::from(spec::FONT_SIZE))
            .selectable(false)
    })
}
