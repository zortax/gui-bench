//! A row of the header scenarios: its label on the left and its value on the right, on one
//! background for every row, or the selection color.

use freya::prelude::*;

use super::{bottom_border, color};
use crate::spec::{self, Row};

/// Returns a 32 px row. The border takes no layout space in freya, so the bottom padding keeps
/// the text centered above it.
pub fn row(text: String, value: u32, selected: bool) -> Rect {
    let background = if selected {
        spec::SELECTED
    } else {
        spec::ROW_EVEN
    };
    rect()
        .horizontal()
        .width(Size::fill())
        .height(Size::px(spec::ROW_HEIGHT))
        .padding(Gaps::new(
            0.0,
            spec::PADDING_X,
            spec::BORDER_HEIGHT,
            spec::PADDING_X,
        ))
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .background(color(background))
        .border(bottom_border(spec::ROW_BORDER))
        .child(label().color(color(spec::LABEL_COLOR)).text(text))
        .child(
            label()
                .color(color(spec::VALUE_COLOR))
                .text(value.to_string()),
        )
}

/// A row that never changes. Its props are the row, so a parent render keeps it as it is.
#[derive(PartialEq)]
pub struct PlainRow {
    row: Row,
    key: DiffKey,
}

impl PlainRow {
    pub fn new(row: Row) -> Self {
        Self {
            row,
            key: DiffKey::None,
        }
    }
}

impl KeyExt for PlainRow {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for PlainRow {
    fn render(&self) -> impl IntoElement {
        row(self.row.label.clone(), self.row.value, false)
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
