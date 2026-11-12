//! Where a run comes from: its revisions, effort, host environment, output and framework
//! versions, and how many of its reports pass their validity checks.

use serde::{Deserialize, Serialize};

/// A source revision that the run records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Revision {
    /// The checkout, such as `gui-bench`, `fakeland` or `zgui`.
    pub name: String,
    /// The short commit, without the dirty mark.
    pub commit: Option<String>,
    /// The checkout had uncommitted changes.
    pub dirty: bool,
}

/// The repeat counts of the measurements.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Effort {
    /// Measured iterations of every measurement.
    pub iterations: Option<u32>,
    /// Hover repeats per iteration.
    pub hover_repeats: Option<u32>,
    /// Repeats per iteration of the table operations and the search probe.
    pub op_repeats: Option<u32>,
    /// Replacements in the churn measurement.
    pub churn_cycles: Option<u32>,
}

/// The host that ran the run, as the bench reports record it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Environment {
    pub cpu: String,
    pub gpu: String,
    pub driver: String,
    pub kernel: String,
    pub governor: String,
    pub renderer: String,
    pub hostname: String,
    pub fakeland_version: String,
}

/// The version of a framework crate that an app builds with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameworkVersion {
    /// The framework, such as `iced`.
    pub framework: String,
    /// The crate, such as `iced` or `eframe`.
    pub krate: String,
    /// The version, such as `0.14.0`.
    pub version: String,
    /// The source, such as `crates.io` or a git URL with its revision.
    pub source: Option<String>,
}

/// The warnings and failures of one validity check over the reports of a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckTally {
    /// The check, such as `wake_lateness`.
    pub check: String,
    /// The reports where the check warns.
    pub warn: u32,
    /// The reports where the check fails.
    pub fail: u32,
}

/// How many bench reports of a run pass their validity checks.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ValidityTally {
    /// Every bench report.
    pub reports: u32,
    /// The reports that count as valid.
    pub valid: u32,
    /// The checks that warn or fail somewhere, worst first.
    pub checks: Vec<CheckTally>,
}

/// Everything known about where a run comes from.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// The start of the run, in Unix seconds.
    pub started_unix: Option<u64>,
    /// The last write of the run index, in Unix seconds.
    pub modified_unix: Option<u64>,
    /// The source revisions.
    pub revisions: Vec<Revision>,
    /// The repeat counts.
    pub effort: Effort,
    /// The output size in pixels.
    pub output: Option<(u32, u32)>,
    /// The output name, make and model of the faked display.
    pub monitor: Option<String>,
    /// The row height in pixels.
    pub row_height: Option<u32>,
    /// The scroll velocity in pixels per second and the glide length in seconds.
    pub scroll: Option<(f64, f64)>,
    /// The ticker interval in milliseconds.
    pub tick_ms: Option<u32>,
    /// The scenarios the run selects.
    pub scenarios: Vec<String>,
    /// The frameworks the run selects.
    pub frameworks: Vec<String>,
    /// The host.
    pub environment: Option<Environment>,
    /// The framework versions. Empty when the run records none.
    pub versions: Vec<FrameworkVersion>,
    /// The validity of the bench reports.
    pub validity: ValidityTally,
    /// The sessions ran an accessibility bus. `None` when the run does not record it.
    pub accessibility: Option<bool>,
    /// Every invocation of the runner that wrote cells of the run, oldest first.
    pub history: Vec<Invocation>,
}

/// One invocation of the runner: a run grows by scenarios when the runner resumes it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Invocation {
    /// The start, in Unix seconds.
    pub started_unix: Option<u64>,
    /// The scenarios it ran.
    pub scenarios: Vec<String>,
    /// The frameworks it ran.
    pub frameworks: Vec<String>,
    /// The source revisions.
    pub revisions: Vec<Revision>,
    /// The sessions ran an accessibility bus.
    pub accessibility: Option<bool>,
}
