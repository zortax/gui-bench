//! Where a run comes from: its revisions, effort, host environment, output, builds and framework
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
    /// Repeats per iteration of the click and key probes.
    pub op_repeats: Option<u32>,
    /// App launches of the startup measurement.
    pub launches: Option<u32>,
    /// How often a measurement runs again when the host disturbs it.
    pub retries: Option<u32>,
}

/// A part of the scenarios that an app builds by hand, since its framework has none.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flag {
    /// The part, such as `menu`, `spinner` or `list`.
    pub part: String,
    /// What the app does, and why.
    pub note: String,
}

/// The build of one framework or edition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Build {
    /// The framework or edition.
    pub framework: String,
    /// The app directory that the build comes from.
    pub app: String,
    /// The size of the release binary that the benchmarks ran, in bytes. It holds line tables.
    pub binary_bytes: Option<u64>,
    /// The size of the size build in bytes: the same app built for the smallest file and
    /// stripped. No benchmark runs it.
    #[serde(default)]
    pub min_binary_bytes: Option<u64>,
    /// The parts the app builds by hand.
    pub flags: Vec<Flag>,
}

/// The scroll inputs of a run: velocities in pixels per second and the length in seconds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScrollSetup {
    pub glide_velocity: Option<f64>,
    pub wheel_velocity: Option<f64>,
    pub fling_velocity: Option<f64>,
    pub seconds: Option<f64>,
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
    /// The refresh rate of every cell and the rate of the second pass of the pacing steps.
    pub hz: Option<(f64, f64)>,
    /// The scroll inputs.
    pub scroll: Option<ScrollSetup>,
    /// The elements of the app without orders, and the elements per order.
    pub elements: Option<(u32, u32)>,
    /// The build of every framework: its binary size and the parts its app builds by hand.
    pub builds: Vec<Build>,
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

impl Provenance {
    /// Returns the refresh rate of every cell.
    pub fn main_hz(&self) -> f64 {
        self.hz.map_or(144.0, |(main, _)| main)
    }

    /// Returns the note of a part that the app of a framework builds by hand.
    pub fn flagged(&self, framework: &str, part: &str) -> Option<&str> {
        self.builds
            .iter()
            .find(|b| b.framework == framework)?
            .flags
            .iter()
            .find(|f| f.part == part)
            .map(|f| f.note.as_str())
    }

    /// Returns the size of the release binary of a framework in bytes.
    pub fn binary_bytes(&self, framework: &str) -> Option<u64> {
        self.builds
            .iter()
            .find(|b| b.framework == framework)?
            .binary_bytes
    }

    /// Returns the size of the size build of a framework in bytes.
    pub fn min_binary_bytes(&self, framework: &str) -> Option<u64> {
        self.builds
            .iter()
            .find(|b| b.framework == framework)?
            .min_binary_bytes
    }

    /// Returns true when the run recorded a size build.
    pub fn has_min_builds(&self) -> bool {
        self.builds.iter().any(|b| b.min_binary_bytes.is_some())
    }
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
