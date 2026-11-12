//! The frame times of one measurement over the cells of one matrix point: the percentiles of the
//! present intervals of every cell, and the late frames of every iteration on the time axis of
//! the iteration.

use serde::{Deserialize, Serialize};

use super::CellId;

/// The percentiles of the present intervals of a measurement, pooled over its iterations, in
/// milliseconds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FrameQuantiles {
    /// The intervals.
    pub count: u32,
    pub mean: f64,
    pub min: f64,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub p999: f64,
    pub max: f64,
}

impl FrameQuantiles {
    /// Returns the percentiles from the median up, with their names.
    pub fn ladder(&self) -> [(&'static str, f64); 6] {
        [
            ("p50", self.p50),
            ("p90", self.p90),
            ("p95", self.p95),
            ("p99", self.p99),
            ("p99.9", self.p999),
            ("max", self.max),
        ]
    }
}

/// A frame that came late, or that left the content where it was.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GapFrame {
    /// The present, in ms from the start of the iteration.
    pub t_ms: f64,
    /// The interval to the previous present.
    pub interval_ms: f64,
    /// The refreshes the frame missed.
    pub missed: u32,
    /// The input moved, but the content stayed (scroll).
    pub stalled: bool,
    /// The content moved by two steps at once (scroll).
    pub double_step: bool,
}

/// One iteration of a measurement on its own time axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GapLane {
    pub index: u32,
    /// The length of the iteration, in ms.
    pub window_ms: f64,
    /// The first and the last present, in ms from the start, when the report has every frame.
    pub presents: Option<(f64, f64)>,
    /// The presented frames.
    pub frames: u32,
    /// The sum of the intervals that the frame statistics count.
    pub total_ms: f64,
    /// `gaps` holds every late frame. Otherwise it holds only the frames over `threshold_ms`.
    pub complete: bool,
    /// The interval above which the report lists a frame.
    pub threshold_ms: f64,
    pub gaps: Vec<GapFrame>,
}

/// The frame times of one cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameProfile {
    pub cell: CellId,
    /// One refresh of the output.
    pub refresh_ms: f64,
    /// The pooled interval percentiles, when the app presented at least two frames.
    pub quantiles: Option<FrameQuantiles>,
    /// The pooled frame rate between the first and the last present.
    pub fps: Option<f64>,
    /// The refreshes without a new frame, over every iteration.
    pub missed_vblanks: u32,
    pub lanes: Vec<GapLane>,
}
