//! The detail of one cell: its outcome, its startup, the validity checks and iterations of each
//! measurement, and the raw series of latency samples, frames, keys, configures and phases.

use serde::{Deserialize, Serialize};

use super::{CellInfo, CheckStatus, Environment};

/// A distribution summary, as fakeland reports write it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub count: u32,
    pub mean: f64,
    pub stddev: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// The startup milestones of the app, in milliseconds from spawn.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Startup {
    pub connected_ms: Option<f64>,
    pub first_surface_ms: Option<f64>,
    pub toplevel_ms: Option<f64>,
    pub first_buffer_ms: Option<f64>,
    pub first_present_ms: Option<f64>,
    /// The app exited before it showed a frame.
    pub exited_early: bool,
}

impl Startup {
    /// Returns the milestones in order, with their names.
    pub fn milestones(&self) -> [(&'static str, Option<f64>); 5] {
        [
            ("connected", self.connected_ms),
            ("first surface", self.first_surface_ms),
            ("toplevel", self.toplevel_ms),
            ("first buffer", self.first_buffer_ms),
            ("first present", self.first_present_ms),
        ]
    }
}

/// One validity check of a report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub value: Option<f64>,
    pub limit: Option<f64>,
}

/// One iteration of a bench, warmups included.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IterationInfo {
    pub index: u32,
    /// The index among the measured iterations; `None` for a warmup.
    pub measured: Option<u32>,
    pub ok: bool,
    pub error: Option<String>,
    /// Compositor wakes past the lateness threshold.
    pub late_wakes: u32,
    /// The largest compositor lateness in milliseconds.
    pub max_lateness_ms: f64,
    /// The iteration length in milliseconds.
    pub duration_ms: f64,
}

/// One bench report of a cell, without its series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementDetail {
    /// The file stem, such as `hover`.
    pub name: String,
    /// The fakeland preset, such as `hover-latency`.
    pub preset: Option<String>,
    pub valid: bool,
    pub checks: Vec<Check>,
    pub iterations: Vec<IterationInfo>,
    /// The preset parameters as `key = value` text.
    pub params: Vec<(String, String)>,
    /// The report sections.
    pub sections: Vec<String>,
}

/// Everything about one cell except the raw series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellDetail {
    pub cell: CellInfo,
    /// The app arguments.
    pub args: Vec<String>,
    pub startup: Option<Startup>,
    pub measurements: Vec<MeasurementDetail>,
    pub environment: Option<Environment>,
    pub has_screenshot: bool,
    pub has_log: bool,
}

/// One latency repeat: input to the first frame that shows the response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatencySample {
    pub repeat: u32,
    /// The input time within the refresh period, in milliseconds.
    pub phase_ms: f64,
    pub total_ms: Option<f64>,
    pub input_to_commit_ms: Option<f64>,
    pub timed_out: bool,
}

/// The latency samples of one measured iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatencyIteration {
    pub index: u32,
    pub samples: Vec<LatencySample>,
}

/// A stage of the latency from input to presentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stage {
    /// The stage, such as `input to commit`.
    pub name: String,
    pub summary: Summary,
}

/// The latency of inputs within one slice of the refresh period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseBin {
    pub from_ms: f64,
    pub to_ms: f64,
    pub count: u32,
    pub total_p50_ms: Option<f64>,
}

/// A histogram bin of the total latency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistogramBin {
    pub lo: f64,
    pub hi: f64,
    pub count: u32,
}

/// The latency series of a measurement in one cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatencyDetail {
    pub measurement: String,
    pub iterations: Vec<LatencyIteration>,
    /// The pooled stages, in order from input to presentation.
    pub stages: Vec<Stage>,
    pub total: Summary,
    pub by_phase: Vec<PhaseBin>,
    pub histogram: Vec<HistogramBin>,
    /// The 95 % CI of the pooled median total latency.
    pub median_ci: Option<(f64, f64)>,
    /// The refresh period of the output in milliseconds.
    pub refresh_ms: f64,
}

/// One presented frame of a scroll glide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameStep {
    /// The presentation time from the first step, in milliseconds.
    pub t_ms: f64,
    pub interval_ms: f64,
    pub expected_px: f64,
    pub measured_px: f64,
    pub stalled: bool,
    pub double_step: bool,
    pub vblanks_missed: u32,
}

/// The frames of one measured scroll iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameIteration {
    pub index: u32,
    pub steps: Vec<FrameStep>,
    pub smoothness_score: Option<f64>,
    pub start_latency_ms: Option<f64>,
}

/// The frame pacing of one measured iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PacingIteration {
    pub index: u32,
    pub fps: f64,
    pub interval: Summary,
    /// Frames per second of the iteration.
    pub fps_series: Vec<u32>,
    pub jank_count: u32,
    pub jank_threshold_ms: f64,
    pub missed_vblanks: u32,
    pub discarded: u32,
}

/// The frame series of a measurement in one cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FramesDetail {
    pub measurement: String,
    /// The scroll steps, empty for measurements without a glide.
    pub scroll: Vec<FrameIteration>,
    pub pacing: Vec<PacingIteration>,
    pub refresh_ms: f64,
}

/// One typed key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyRecord {
    /// The send time from the first key, in milliseconds.
    pub t_ms: f64,
    pub key: String,
    pub latency_ms: Option<f64>,
    pub handling_ms: Option<f64>,
    pub backlog: u32,
}

/// The keys of one measured typing iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypingIteration {
    pub index: u32,
    pub keys: Vec<KeyRecord>,
    pub latency_slope_ms_per_key: Option<f64>,
}

/// The typing series of a measurement in one cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypingDetail {
    pub measurement: String,
    pub iterations: Vec<TypingIteration>,
    pub refresh_ms: f64,
}

/// One configure of a resize or scale switch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureRecord {
    /// The send time from the first configure, in milliseconds.
    pub t_ms: f64,
    pub width: u32,
    pub height: u32,
    pub ack_ms: Option<f64>,
    pub commit_ms: Option<f64>,
    pub present_ms: Option<f64>,
    pub superseded: bool,
}

/// The configures of one measured iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResizeIteration {
    pub index: u32,
    pub configures: Vec<ConfigureRecord>,
}

/// The resize series of a measurement in one cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResizeDetail {
    pub measurement: String,
    pub iterations: Vec<ResizeIteration>,
    pub refresh_ms: f64,
}

/// The durations of one app phase from zgui's app marks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseSpan {
    /// The phase, such as `layout` or `paint`.
    pub name: String,
    pub count: u32,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub mean_ms: f64,
    pub max_ms: f64,
    pub durations_ms: Vec<f64>,
}

/// The app phases of a measurement in one cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhasesDetail {
    pub measurement: String,
    pub phases: Vec<PhaseSpan>,
}
