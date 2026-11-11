//! The spinner scenario: a header whose spinner square turns once a second, above static rows.
//! GPUI rotates only `svg()` elements (`Transformation`), so the square is an inline SVG, turned
//! by a repeating linear `with_animation` the way gpui-component's own `Spinner` turns its icon.

use std::time::Duration;

use gpui_kit::*;

use crate::page::{header, list, page, plain_row};
use crate::spec::{self, Ids, Row};

/// A rounded square that fills its box; `text_color` paints it.
const SQUARE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect width="24" height="24" rx="3" ry="3"/></svg>"#;

/// The page with static rows and the spinner.
pub struct Spinner {
    rows: Vec<Row>,
}

impl Spinner {
    pub fn new(count: usize) -> Self {
        Self {
            rows: Ids::default().rows(count),
        }
    }
}

impl Render for Spinner {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let spinner = svg()
            .data(SQUARE)
            .flex_none()
            .size(px(spec::SPINNER_SIZE))
            .text_color(rgb(spec::SPINNER_COLOR))
            .with_animation(
                "spin",
                Animation::new(Duration::from_millis(spec::SPINNER_PERIOD_MS)).repeat(),
                |square, delta| {
                    square.with_transformation(Transformation::rotate(percentage(delta)))
                },
            );
        let rows = self
            .rows
            .iter()
            .map(|row| plain_row(row.label.clone(), row.value, false));
        page()
            .child(header(spec::status(self.rows.len())).child(spinner))
            .child(list().children(rows))
    }
}
