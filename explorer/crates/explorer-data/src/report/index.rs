//! The run index `run.json`, the cell outcome `cell.json` and the startup report.

use std::collections::BTreeMap;

use explorer_core::api::CellKey;
use serde::Deserialize;

/// The run index, format 2.
#[derive(Debug, Clone, Deserialize)]
pub struct RunIndex {
    pub format: Option<u32>,
    pub started_unix: Option<u64>,
    #[serde(default)]
    pub effort: BTreeMap<String, serde_json::Value>,
    pub output: Option<Output>,
    pub row_height: Option<u32>,
    pub scroll: Option<Scroll>,
    pub tick_ms: Option<u32>,
    #[serde(default)]
    pub revisions: BTreeMap<String, Option<String>>,
    /// Framework versions: `{framework: {crate, version, source}}`. Runs record it from a later
    /// runner revision on.
    #[serde(default)]
    pub framework_versions: BTreeMap<String, VersionRaw>,
    #[serde(default)]
    pub cells: Vec<CellOutcome>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Output {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scroll {
    pub velocity: f64,
    pub seconds: f64,
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct CellRaw {
    pub scenario: String,
    pub framework: String,
    pub variant: String,
    pub hz: f64,
    pub rows: u32,
    pub updates: Option<u32>,
}

impl CellRaw {
    pub fn key(&self) -> CellKey {
        CellKey {
            scenario: self.scenario.clone(),
            framework: self.framework.clone(),
            variant: self.variant.clone(),
            hz: self.hz,
            rows: self.rows,
            updates: self.updates,
        }
    }
}

/// The startup report: spawn to the first presented frame.
#[derive(Debug, Clone, Deserialize)]
pub struct Startup {
    pub connected_ms: Option<f64>,
    pub first_surface_ms: Option<f64>,
    pub toplevel_ms: Option<f64>,
    pub first_buffer_ms: Option<f64>,
    pub first_present_ms: Option<f64>,
    #[serde(default)]
    pub exited_early: bool,
}

impl Startup {
    pub fn to_api(&self) -> explorer_core::api::Startup {
        explorer_core::api::Startup {
            connected_ms: self.connected_ms,
            first_surface_ms: self.first_surface_ms,
            toplevel_ms: self.toplevel_ms,
            first_buffer_ms: self.first_buffer_ms,
            first_present_ms: self.first_present_ms,
            exited_early: self.exited_early,
        }
    }
}
