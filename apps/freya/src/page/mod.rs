//! The page of the header scenarios: a header with the status text and a slot on the right, and
//! a scroll view of rows below it. Rows here have one background and no hover color. Also holds
//! the window that every new scenario opens.

mod header;
mod row;

use freya::prelude::*;

use crate::spec;

pub use header::header;
pub use row::{PlainRow, row};

const APP_ID: &str = "gui-bench.freya";

/// Returns the freya color of a spec color.
pub const fn color(hex: u32) -> Color {
    let (r, g, b) = spec::rgb(hex);
    Color::from_rgb(r, g, b)
}

/// Opens an undecorated 1280x800 window with `app` as its root and runs the event loop until it
/// closes.
pub fn launch_app(title: &'static str, app: impl App) {
    let window = WindowConfig::new_app(app)
        .with_title(title)
        .with_app_id(APP_ID)
        .with_size(f64::from(spec::WINDOW.0), f64::from(spec::WINDOW.1))
        .with_decorations(false)
        .with_background(color(spec::BACKGROUND));
    launch(LaunchConfig::new().with_window(window));
}

/// Returns the page: the header above a scroll view that holds `rows`.
pub fn page(header: impl IntoElement, rows: impl IntoIterator<Item = Element>) -> Rect {
    rect()
        .expanded()
        .background(color(spec::BACKGROUND))
        .font_family(spec::FONT_FAMILY)
        .font_size(spec::FONT_SIZE)
        .child(header)
        .child(ScrollView::new().children(rows))
}

/// Returns a border of `fill` on the bottom edge only, inside the element.
pub fn bottom_border(fill: u32) -> Border {
    Border::new()
        .fill(color(fill))
        .width(BorderWidth {
            top: 0.0,
            right: 0.0,
            bottom: spec::BORDER_HEIGHT,
            left: 0.0,
        })
        .alignment(BorderAlignment::Inner)
}
