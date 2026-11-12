//! The geometry of the dashboard scenario, mirrored from `apps/shared/dashboard.rs`: the points
//! and colors that the layout check reads, and the elements that the probes hover, click and
//! scroll. Coordinates are window pixels at scale 1 before any scroll.

use serde_json::{Value, json};

/// Where the pointer rests for the layout check: the empty top bar between the title and the
/// search input.
pub const PARK: (i32, i32) = (500, 32);

/// The points the layout check reads and their colors: the sidebar, the top bar, the padding of
/// the content, a metric card, the chart card, the header of the orders, a plain order row, a
/// plain nav item and the active nav item.
pub const CHECKS: [(i32, i32, u32); 9] = [
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

/// The top of the first order row.
pub const TABLE_ROWS_TOP: i32 = 596;
/// The height of an order row.
const ORDER_ROW_HEIGHT: i32 = 52;
/// The left edge and the width of the content cards.
const CARD_X: i32 = 264;
const CARD_WIDTH: i32 = 992;

/// A probe of one element: where the pointer goes, where it rests between repeats, and the region
/// whose pixels show the response.
#[derive(Debug, Clone)]
pub struct Target {
    pub at: (i32, i32),
    pub away: (i32, i32),
    pub roi: Value,
}

/// A click probe of a toggle: where the click lands, and the region whose pixels show the
/// response.
#[derive(Debug, Clone)]
pub struct Toggle {
    pub at: (i32, i32),
    pub roi: Value,
}

/// Returns the region of a rectangle.
fn roi(x: i32, y: i32, w: i32, h: i32) -> Value {
    json!({ "x": x, "y": y, "w": w, "h": h })
}

/// The pointer enters nav item 3 ("Products") from nav item 6; the item takes the hover color.
pub fn hover_nav() -> Target {
    let item = |i: i32| 80 + 44 * i;
    Target {
        at: (120, item(3) + 20),
        away: (120, item(6) + 20),
        roi: roi(12, item(3), 216, 40),
    }
}

/// The pointer enters order row 1 from row 3; the row takes the hover color.
pub fn hover_row() -> Target {
    let row = |r: i32| TABLE_ROWS_TOP + r * ORDER_ROW_HEIGHT;
    Target {
        at: (760, row(1) + ORDER_ROW_HEIGHT / 2),
        away: (760, row(3) + ORDER_ROW_HEIGHT / 2),
        roi: roi(CARD_X, row(1), CARD_WIDTH, ORDER_ROW_HEIGHT),
    }
}

/// A click on the compare switch shows the previous-period bars of the chart; the next click
/// hides them.
pub fn click_compare() -> Toggle {
    Toggle {
        at: (1216, 426),
        roi: roi(284, 288, 600, 228),
    }
}

/// A click on the "Refunded" checkbox hides the refunded orders, the first of them in row 1; the
/// next click shows them.
pub fn click_filter() -> Toggle {
    Toggle {
        at: (949, 384),
        roi: roi(
            CARD_X,
            TABLE_ROWS_TOP + ORDER_ROW_HEIGHT,
            CARD_WIDTH,
            800 - TABLE_ROWS_TOP - ORDER_ROW_HEIGHT,
        ),
    }
}

/// The pointer of the scroll glide over the orders, and the region that motion estimation reads:
/// the content above the order row under the pointer, so its hover color stays outside.
pub const SCROLL_POINTER: (i32, i32) = (760, 720);
pub fn scroll_roi() -> Value {
    roi(CARD_X, 88, CARD_WIDTH, 552)
}
