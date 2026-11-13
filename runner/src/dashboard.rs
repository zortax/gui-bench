//! The geometry of the scenarios, mirrored from `apps/shared/dashboard.rs`: the points and colors
//! that the layout check reads, and the elements that the probes hover, click and scroll.
//! Coordinates are window pixels at scale 1 before any scroll.

use serde_json::{Value, json};

use crate::scenarios::Kind;

/// The output size in logical pixels.
pub const OUTPUT: (i32, i32) = (1280, 800);

/// The elements of the app outside the orders, and of one order row, as the spec counts them.
pub const CHROME_ELEMENTS: u32 = 228;
pub const ORDER_ELEMENTS: u32 = 11;

/// Where the pointer rests for the layout check and between measurements: the empty top bar
/// between the title and the search input.
pub const PARK: (i32, i32) = (500, 32);

/// The points the layout check of the app reads and their colors: the sidebar, the top bar, the
/// padding of the content, a metric card, the chart card, the header of the orders, a plain order
/// row, a plain nav item and the active nav item.
const APP_CHECKS: [(i32, i32, u32); 9] = [
    (120, 600, 0x111827),
    (600, 10, 0xffffff),
    (252, 400, 0xf3f4f6),
    (700, 104, 0xffffff),
    (500, 250, 0xffffff),
    (1100, 574, 0xf9fafb),
    (1060, 674, 0xffffff),
    (200, 232, 0x111827),
    (200, 100, 0x1f2937),
];

/// The points the layout check of the list reads: the header, two plain rows between the status
/// and the progress columns, and the rule under the first row.
const LIST_CHECKS: [(i32, i32, u32); 4] = [
    (1100, 22, 0xf9fafb),
    (560, 70, 0xffffff),
    (560, 122, 0xffffff),
    (560, 95, 0xf3f4f6),
];

/// The top of the first order row of the app.
const TABLE_ROWS_TOP: i32 = 596;
/// The top of the first row of the list, under its header.
const LIST_ROWS_TOP: i32 = 44;
/// The height of an order row.
const ORDER_ROW_HEIGHT: i32 = 52;
/// The left edge and the width of the content cards.
const CARD_X: i32 = 264;
const CARD_WIDTH: i32 = 992;
/// The top of the content, under the top bar.
const CONTENT_TOP: i32 = 64;

/// A probe of one element: where the pointer goes, where it rests between repeats, and the region
/// whose pixels show the response.
#[derive(Debug, Clone)]
pub struct Target {
    pub at: (i32, i32),
    pub away: (i32, i32),
    pub roi: Value,
}

/// A click probe: where the click lands, where the click between repeats lands when another
/// element brings the state back, and the region whose pixels show the response.
#[derive(Debug, Clone)]
pub struct Toggle {
    pub at: (i32, i32),
    pub undo: Option<(i32, i32)>,
    pub roi: Value,
}

/// Returns the region of a rectangle.
fn roi(x: i32, y: i32, w: i32, h: i32) -> Value {
    json!({ "x": x, "y": y, "w": w, "h": h })
}

/// Returns the points and colors of the layout check of a scenario.
pub fn checks(kind: Kind) -> &'static [(i32, i32, u32)] {
    match kind {
        Kind::App => &APP_CHECKS,
        Kind::List => &LIST_CHECKS,
    }
}

/// Returns the window y of the top of nav item `i`.
fn nav_item(i: i32) -> i32 {
    80 + 44 * i
}

/// The pointer enters nav item 3 ("Products") from nav item 5; the item takes the hover color.
pub fn hover_nav() -> Target {
    Target {
        at: (120, nav_item(3) + 20),
        away: (120, nav_item(5) + 20),
        roi: roi(12, nav_item(3), 216, 40),
    }
}

/// The pointer enters order row 1 from row 3; the row takes the hover color. `top` is the top of
/// the first row and the row spans `x..x + width`.
fn hover_order(top: i32, x: i32, width: i32) -> Target {
    let row = |r: i32| top + r * ORDER_ROW_HEIGHT;
    Target {
        at: (760, row(1) + ORDER_ROW_HEIGHT / 2),
        away: (760, row(3) + ORDER_ROW_HEIGHT / 2),
        roi: roi(x, row(1), width, ORDER_ROW_HEIGHT),
    }
}

/// The hover of an order row of the app.
pub fn hover_row() -> Target {
    hover_order(TABLE_ROWS_TOP, CARD_X, CARD_WIDTH)
}

/// The hover of a row of the list.
pub fn hover_list_row() -> Target {
    hover_order(LIST_ROWS_TOP, 0, OUTPUT.0)
}

/// A click on the compare switch shows the previous-period bars of the chart; the next click
/// hides them.
pub fn click_compare() -> Toggle {
    Toggle {
        at: (1216, 426),
        undo: None,
        roi: roi(284, 288, 600, 228),
    }
}

/// The order rows of the app from row `first` to the bottom of the window.
fn orders_roi(first: i32) -> Value {
    let top = TABLE_ROWS_TOP + first * ORDER_ROW_HEIGHT;
    roi(CARD_X, top, CARD_WIDTH, OUTPUT.1 - top)
}

/// A click on the "Refunded" checkbox hides the refunded orders, the first of them in row 1; the
/// next click shows them.
pub fn click_filter() -> Toggle {
    Toggle {
        at: (949, 384),
        undo: None,
        roi: orders_roi(1),
    }
}

/// A click on the "AMOUNT" column title sorts the orders by amount; the next click brings the
/// order of the data back.
pub fn click_sort() -> Toggle {
    Toggle {
        at: (CARD_X + 848 + 90, 552 + 22),
        undo: None,
        roi: orders_roi(0),
    }
}

/// The same title in the list, where it ends 24 px from the right edge of the window.
pub fn click_list_sort() -> Toggle {
    Toggle {
        at: (OUTPUT.0 - 24 - 30, 22),
        undo: None,
        roi: roi(0, LIST_ROWS_TOP, OUTPUT.0, OUTPUT.1 - LIST_ROWS_TOP),
    }
}

/// A click on the export button opens its menu over the content; the next click closes it.
pub fn click_menu() -> Toggle {
    Toggle {
        at: (1148, 32),
        undo: None,
        roi: roi(1100, CONTENT_TOP - 10, 160, 126),
    }
}

/// The content under the top bar, which a page switch replaces.
fn content_roi() -> Value {
    roi(240, CONTENT_TOP, OUTPUT.0 - 240, OUTPUT.1 - CONTENT_TOP)
}

/// The center of nav item `i`.
fn nav_center(i: i32) -> (i32, i32) {
    (120, nav_item(i) + 20)
}

/// The nav items of the overview and of the messages page.
pub const NAV_OVERVIEW: i32 = 0;
pub const NAV_MESSAGES: i32 = 6;

/// A click on nav item `to` shows its page; a click on nav item `from` between repeats goes back.
pub fn click_page(from: i32, to: i32) -> Toggle {
    Toggle {
        at: nav_center(to),
        undo: Some(nav_center(from)),
        roi: content_roi(),
    }
}

/// The center of nav item `i`, for a plain click.
pub fn nav(i: i32) -> (i32, i32) {
    nav_center(i)
}

/// The search input of the top bar, and its region.
pub const SEARCH: (i32, i32) = (816, 32);
pub fn search_roi() -> Value {
    roi(676, 14, 280, 36)
}

/// The order rows that the search filters.
pub fn filtered_roi() -> Value {
    orders_roi(0)
}

/// The live switch of the top bar.
pub const LIVE: (i32, i32) = (996, 32);

/// The pointer of the scroll over the orders of the app, and the region that motion estimation
/// reads: the content above the order row under the pointer, so its hover color stays outside.
pub const SCROLL_POINTER: (i32, i32) = (760, 720);
pub fn scroll_roi() -> Value {
    roi(CARD_X, 88, CARD_WIDTH, 552)
}

/// The same for the list: the rows above the row under the pointer.
pub fn list_scroll_roi() -> Value {
    roi(0, LIST_ROWS_TOP, OUTPUT.0, 596)
}

/// Returns true when `rows` order rows are tall enough for `travel` pixels of scroll in both
/// directions.
pub fn scrolls(rows: u32, travel: f64) -> bool {
    f64::from(rows) * f64::from(ORDER_ROW_HEIGHT) >= f64::from(OUTPUT.1) + travel * 1.5
}
