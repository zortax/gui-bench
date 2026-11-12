//! The window geometry that every app shares, mirrored from `apps/shared/spec.rs`: the output
//! size, the row and header heights, the colors the layout check reads, and the regions that the
//! probes watch.
//!
//! The `rows` and `virtual` scenarios put row `i` at `i * ROW_HEIGHT` from the top of the window.
//! The header scenarios put a header of `HEADER_HEIGHT` above the rows, so row `i` starts at
//! `HEADER_HEIGHT + i * ROW_HEIGHT`.

use serde_json::{Value, json};

/// The output size in logical pixels.
pub const OUTPUT: (i32, i32) = (1280, 800);
/// The height of one row in logical pixels.
pub const ROW_HEIGHT: i32 = 32;
/// The height of the header above the rows in the header scenarios.
pub const HEADER_HEIGHT: i32 = 40;

/// Colors as `0xrrggbb`, the same values as the apps use.
pub const ROW_EVEN: u32 = 0xffffff;
pub const ROW_ODD: u32 = 0xf6f8fa;
pub const ROW_BORDER: u32 = 0xe5e7eb;
pub const HEADER_BG: u32 = 0xeef1f4;

/// The layout of a scenario: rows from the top of the window, rows under the header, or the
/// dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// Rows from the top of the window, with stripes and a hover color.
    Rows,
    /// A header, then rows with one background and no hover color.
    Header,
    /// The dashboard of `apps/shared/dashboard.rs`; see `crate::dashboard`.
    Dashboard,
}

impl Page {
    /// Returns the window y of the top of the first row.
    pub fn top(self) -> i32 {
        match self {
            Self::Rows => 0,
            Self::Header => HEADER_HEIGHT,
            Self::Dashboard => crate::dashboard::TABLE_ROWS_TOP,
        }
    }

    /// Returns where the pointer rests while the layout check reads the window: the bottom of the
    /// window, away from the rows it reads, or the empty part of the dashboard's top bar.
    pub fn park(self) -> (i32, i32) {
        match self {
            Self::Rows | Self::Header => (OUTPUT.0 / 2, OUTPUT.1 - 8),
            Self::Dashboard => crate::dashboard::PARK,
        }
    }

    /// Returns the window y of the vertical center of a row.
    pub fn row_center(self, row: i32) -> i32 {
        self.top() + row * ROW_HEIGHT + ROW_HEIGHT / 2
    }

    /// Returns the region of `n` rows from row `first`, over the full width.
    pub fn rows_roi(self, first: i32, n: i32) -> Value {
        json!({ "x": 0, "y": self.top() + first * ROW_HEIGHT, "w": OUTPUT.0, "h": n * ROW_HEIGHT })
    }

    /// Returns the points and colors that the layout check reads: plain background between the
    /// label and the value of the first rows, and the bottom border of the first row. A list of
    /// fewer than two rows checks the header and the empty list only.
    pub fn checks(self, rows: u32) -> Vec<(i32, i32, u32)> {
        let x = OUTPUT.0 / 2;
        if self == Self::Dashboard {
            return crate::dashboard::CHECKS.to_vec();
        }
        if rows < 2 {
            return match self {
                Self::Rows => vec![(x, OUTPUT.1 - 16, ROW_EVEN)],
                Self::Header | Self::Dashboard => vec![
                    (x, HEADER_HEIGHT / 2, HEADER_BG),
                    (x, OUTPUT.1 - 16, ROW_EVEN),
                ],
            };
        }
        match self {
            Self::Rows => vec![
                (x, self.row_center(0), ROW_EVEN),
                (x, self.row_center(1), ROW_ODD),
                (x, ROW_HEIGHT - 1, ROW_BORDER),
            ],
            Self::Header | Self::Dashboard => vec![
                (x, HEADER_HEIGHT / 2, HEADER_BG),
                (x, self.row_center(0), ROW_EVEN),
                (x, self.row_center(1), ROW_EVEN),
                (x, HEADER_HEIGHT + ROW_HEIGHT - 1, ROW_BORDER),
            ],
        }
    }
}

/// Returns the region of the header's status text.
pub fn status_roi() -> Value {
    json!({ "x": 0, "y": 0, "w": OUTPUT.0 / 2, "h": HEADER_HEIGHT - 1 })
}
