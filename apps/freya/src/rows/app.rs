//! The rows window: an undecorated window whose root is one vertical scroll view with all rows.

use freya::prelude::*;

use super::row::Row;

const APP_ID: &str = "gui-bench.freya";
const TITLE: &str = "gui-bench rows";
const SIZE: (f64, f64) = (1280.0, 800.0);
const FONT_FAMILY: &str = "sans-serif";
const FONT_SIZE: f32 = 14.0;
const BACKGROUND: Color = Color::from_rgb(0xff, 0xff, 0xff);

/// Opens the window with `count` rows and runs the event loop until it closes.
pub fn run(count: usize) {
    let window = WindowConfig::new_app(RowsApp { count })
        .with_title(TITLE)
        .with_app_id(APP_ID)
        .with_size(SIZE.0, SIZE.1)
        .with_decorations(false)
        .with_background(BACKGROUND);
    launch(LaunchConfig::new().with_window(window));
}

/// The root component: the scroll view and its rows.
struct RowsApp {
    count: usize,
}

impl App for RowsApp {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .background(BACKGROUND)
            .font_family(FONT_FAMILY)
            .font_size(FONT_SIZE)
            .child(
                ScrollView::new().child(
                    rect()
                        .width(Size::fill())
                        .children((0..self.count).map(Row::new)),
                ),
            )
    }
}
