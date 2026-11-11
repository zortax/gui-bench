//! The page of the header scenarios: a header with the status text and a slot on the right, and
//! a scroll container of rows below it. Rows here have one background and no hover color.

mod header;
mod row;

use gpui_kit::component::v_flex;
use gpui_kit::*;

pub use self::header::header;
pub use self::row::plain_row;
use crate::spec;

/// Returns the page: a column that fills the window, for a header and a list.
pub fn page() -> Div {
    v_flex()
        .size_full()
        .bg(rgb(spec::BACKGROUND))
        .text_size(px(spec::FONT_SIZE))
}

/// Returns the scroll container below the header, for the rows.
pub fn list() -> Stateful<Div> {
    v_flex()
        .id("list")
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scroll()
        .bg(rgb(spec::BACKGROUND))
}
