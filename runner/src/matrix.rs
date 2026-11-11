//! The benchmark matrix: frameworks, refresh rates and row counts, and the scenario parameters
//! that every cell shares. Window coordinates follow the rows app layout: row `i` spans
//! `i * ROW_HEIGHT .. (i + 1) * ROW_HEIGHT` from the top of the window.

use fakeland_bench::PresetParams;
use serde::Serialize;
use serde_json::json;

/// The output size in logical pixels.
pub const OUTPUT: (i32, i32) = (1280, 800);
/// The height of one row in logical pixels.
pub const ROW_HEIGHT: i32 = 32;
/// The row that the hover probe enters.
pub const HOVER_ROW: i32 = 5;
/// The row that the pointer rests on between hover repeats.
pub const AWAY_ROW: i32 = 1;
/// The scroll velocity in logical pixels per second.
pub const SCROLL_VELOCITY: f64 = 800.0;
/// The length of one scroll glide in seconds.
pub const SCROLL_SECONDS: f64 = 1.0;
/// The window y of the pointer during a glide. The row under the pointer shows its hover color,
/// which stays at the pointer while the content moves.
const SCROLL_POINTER_Y: i32 = 720;
/// The height of the region that motion estimation reads, from the top of the window. It ends a
/// row above the pointer, so the hover color stays outside it.
const SCROLL_ROI_HEIGHT: i32 = 672;

/// One cell of the matrix.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Cell {
    /// The refresh rate of the output in Hz.
    pub hz: f64,
    /// The number of rows the app renders.
    pub rows: u32,
}

/// The repeat counts of the scenarios.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Effort {
    /// Hover repeats per iteration.
    pub hover_repeats: u32,
    /// Measured iterations of every scenario.
    pub iterations: u32,
}

impl Cell {
    /// Returns the window y of the vertical center of a row.
    fn row_center(row: i32) -> i32 {
        row * ROW_HEIGHT + ROW_HEIGHT / 2
    }

    /// Returns true when the list is tall enough for a glide in both directions.
    pub fn scrolls(&self) -> bool {
        let travel = SCROLL_VELOCITY * SCROLL_SECONDS;
        f64::from(self.rows) * f64::from(ROW_HEIGHT) >= f64::from(OUTPUT.1) + travel * 1.5
    }

    /// Returns the hover-latency preset parameters with a detector: `pixels` or `app_mark`.
    pub fn hover_params(&self, detector: &str, effort: Effort) -> PresetParams {
        let x = OUTPUT.0 / 2;
        PresetParams::new()
            .set(
                "position",
                json!({ "x": x, "y": Self::row_center(HOVER_ROW) }),
            )
            .set("away", json!({ "x": x, "y": Self::row_center(AWAY_ROW) }))
            .set(
                "roi",
                json!({ "x": 0, "y": HOVER_ROW * ROW_HEIGHT, "w": OUTPUT.0, "h": ROW_HEIGHT }),
            )
            .set("detector", detector)
            .set("repeats", effort.hover_repeats)
            .set("iterations", effort.iterations)
            .set("warmup", 1)
            .set("timeout", "2s")
    }

    /// Returns the scroll-glide preset parameters. The finger source sends pixel distances with
    /// no kinetic tail. The event rate is four events per refresh, so every frame consumes the
    /// same scroll. Motion estimation reads the rows above the pointer.
    pub fn scroll_params(&self, effort: Effort) -> PresetParams {
        PresetParams::new()
            .set("source", json!({ "type": "finger", "kinetic": false }))
            .set("velocity", SCROLL_VELOCITY)
            .set("duration", format!("{SCROLL_SECONDS}s"))
            .set("rate_hz", self.hz * 4.0)
            .set(
                "position",
                json!({ "x": OUTPUT.0 / 2, "y": SCROLL_POINTER_Y }),
            )
            .set(
                "roi",
                json!({ "x": 0, "y": 0, "w": OUTPUT.0, "h": SCROLL_ROI_HEIGHT }),
            )
            .set("iterations", effort.iterations + 1)
            .set("warmup", 1)
    }

    /// Returns the directory name of the cell, such as `144hz/1000`.
    pub fn dir(&self) -> String {
        format!("{}hz/{}", self.hz, self.rows)
    }
}
