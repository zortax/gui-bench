//! The run index `run.json`, the cell outcome `cell.json` and the startup report. Serde skips
//! unknown fields, and every field beyond the cell identity is optional.

use std::collections::BTreeMap;

use explorer_core::api::{CellKey, Launch, Part};
use serde::Deserialize;

/// The run index, format 3.
#[derive(Debug, Clone, Deserialize)]
pub struct RunIndex {
    pub format: Option<u32>,
    pub started_unix: Option<u64>,
    #[serde(default)]
    pub effort: BTreeMap<String, serde_json::Value>,
    pub output: Option<Output>,
    pub hz: Option<Rates>,
    pub scroll: Option<Scroll>,
    pub elements: Option<Elements>,
    #[serde(default)]
    pub builds: BTreeMap<String, BuildRaw>,
    #[serde(default)]
    pub revisions: BTreeMap<String, Option<String>>,
    /// Framework versions: `{framework: {crate, version, source}}`.
    #[serde(default)]
    pub framework_versions: BTreeMap<String, VersionRaw>,
    #[serde(default)]
    pub cells: Vec<CellOutcome>,
    pub accessibility: Option<bool>,
    #[serde(default)]
    pub history: Vec<InvocationRaw>,
}

/// One invocation of the runner in the history of a run.
#[derive(Debug, Clone, Deserialize)]
pub struct InvocationRaw {
    pub started_unix: Option<u64>,
    #[serde(default)]
    pub scenarios: Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
    #[serde(default)]
    pub revisions: BTreeMap<String, Option<String>>,
    pub accessibility: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Output {
    pub width: u32,
    pub height: u32,
}

/// The refresh rate of every cell and the rate of the second pass of the pacing steps.
#[derive(Debug, Clone, Deserialize)]
pub struct Rates {
    pub main: f64,
    pub slow: Option<f64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Scroll {
    pub glide_velocity: Option<f64>,
    pub wheel_velocity: Option<f64>,
    pub fling_velocity: Option<f64>,
    pub seconds: Option<f64>,
}

/// The elements of the app without orders, and the elements per order.
#[derive(Debug, Clone, Deserialize)]
pub struct Elements {
    pub chrome: u32,
    pub per_row: u32,
}

/// The build of a framework: its app, the size of its binary and the parts built by hand.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct BuildRaw {
    pub app: Option<String>,
    pub binary_bytes: Option<u64>,
    pub min_binary_bytes: Option<u64>,
    pub flags: Vec<FlagRaw>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FlagRaw {
    pub part: String,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionRaw {
    #[serde(rename = "crate")]
    pub krate: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
}

/// The outcome of one cell, as `cell.json` and the run index hold it.
#[derive(Debug, Clone, Deserialize)]
pub struct CellOutcome {
    pub cell: CellRaw,
    pub dir: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub done: Vec<String>,
    #[serde(default)]
    pub errors: Vec<(String, String)>,
    #[serde(default)]
    pub skipped: Vec<(String, String)>,
    #[serde(default)]
    pub missed_refreshes: bool,
    #[serde(default)]
    pub over_budget: bool,
    #[serde(default)]
    pub stopped: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CellRaw {
    pub scenario: String,
    pub framework: String,
    pub hz: f64,
    pub rows: u32,
    #[serde(default)]
    pub elements: Option<u32>,
    #[serde(default)]
    pub part: Option<String>,
}

impl CellRaw {
    pub fn key(&self) -> CellKey {
        CellKey {
            scenario: self.scenario.clone(),
            framework: self.framework.clone(),
            hz: self.hz,
            rows: self.rows,
            elements: self.elements,
        }
    }

    pub fn part(&self) -> Part {
        self.part.as_deref().map_or(Part::Full, Part::parse)
    }
}

/// The startup report: one entry per launch.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Startup {
    pub launches: Vec<LaunchRaw>,
}

/// The milestones of one launch, from spawn.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LaunchRaw {
    pub connected_ms: Option<f64>,
    pub first_surface_ms: Option<f64>,
    pub toplevel_ms: Option<f64>,
    pub first_buffer_ms: Option<f64>,
    pub first_present_ms: Option<f64>,
    pub content_ms: Option<f64>,
    pub exited_early: bool,
}

impl LaunchRaw {
    /// Returns the milestones with their metric names.
    pub fn milestones(&self) -> [(&'static str, Option<f64>); 6] {
        [
            ("connected_ms", self.connected_ms),
            ("first_surface_ms", self.first_surface_ms),
            ("toplevel_ms", self.toplevel_ms),
            ("first_buffer_ms", self.first_buffer_ms),
            ("first_present_ms", self.first_present_ms),
            ("content_ms", self.content_ms),
        ]
    }

    pub fn to_api(&self) -> Launch {
        Launch {
            connected_ms: self.connected_ms,
            first_surface_ms: self.first_surface_ms,
            toplevel_ms: self.toplevel_ms,
            first_buffer_ms: self.first_buffer_ms,
            first_present_ms: self.first_present_ms,
            content_ms: self.content_ms,
            exited_early: self.exited_early,
        }
    }
}
