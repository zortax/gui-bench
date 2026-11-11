//! A row of the header scenarios: a 32 px band with its label on the left and its value on the
//! right, white, with no hover color.

use floem::prelude::*;
use floem::style::Style;

use super::color;
use crate::spec::{self, Row};

/// Returns the band of a row around its label and value.
pub fn band(label: Label, value: Label) -> Stack {
    let label = label.style(|s| s.color(color(spec::LABEL_COLOR)));
    let value = value.style(|s| s.color(color(spec::VALUE_COLOR)));
    Stack::horizontal((label, value)).style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(f64::from(spec::ROW_HEIGHT))
            .padding_horiz(f64::from(spec::PADDING_X))
            .items_center()
            .justify_between()
            .border_bottom(f64::from(spec::BORDER_HEIGHT))
            .border_color(color(spec::ROW_BORDER))
            .background(color(spec::ROW_EVEN))
    })
}

/// Gives a row the selected background when `selected`.
pub fn selected_style(s: Style, selected: bool) -> Style {
    s.apply_if(selected, |s| s.background(color(spec::SELECTED)))
}

/// Returns a row that never changes.
pub fn plain_row(row: Row) -> Stack {
    band(Label::new(row.label), Label::new(row.value))
}

/// Returns a vertical stack of rows that never change.
pub fn plain_rows(rows: Vec<Row>) -> Stack {
    Stack::vertical_from_iter(rows.into_iter().map(plain_row)).style(|s| s.width_full())
}
