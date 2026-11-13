//! A matrix cell: its key (scenario, framework, refresh rate, size), its outcome and the health
//! of its measurements.

use serde::{Deserialize, Serialize};

/// The index of a cell in its run, stable for one load of the run.
pub type CellId = u32;

/// One point of the benchmark matrix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellKey {
    /// The scenario, `app` or `list`.
    pub scenario: String,
    /// The framework or edition, such as `zgui` or `iced-tuned`.
    pub framework: String,
    /// The refresh rate of the output in Hz.
    pub hz: f64,
    /// The number of rows the app holds: orders and messages, or rows of list data.
    pub rows: u32,
    /// The elements of the app as the spec counts them. A list has none.
    pub elements: Option<u32>,
}

impl CellKey {
    /// Returns the size on a size axis: the elements of the app, or the rows of the list.
    pub fn size(&self) -> f64 {
        f64::from(self.elements.unwrap_or(self.rows))
    }

    /// Returns the size as text, such as `11,228 elements (1,000 orders)` or `10,000 rows`.
    pub fn size_text(&self) -> String {
        let rows = crate::units::thousands(f64::from(self.rows));
        match self.elements {
            Some(e) => format!(
                "{} elements ({rows} orders)",
                crate::units::thousands(f64::from(e))
            ),
            None => format!("{rows} rows"),
        }
    }

    /// Returns true when both keys name the same scenario, refresh rate and size.
    pub fn same_point(&self, other: &CellKey) -> bool {
        self.scenario == other.scenario && self.hz == other.hz && self.rows == other.rows
    }
}

/// Which measurements a cell takes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Part {
    /// Every measurement of the scenario.
    #[default]
    Full,
    /// The pacing measurements only, at the slow refresh rate.
    Pacing,
}

impl Part {
    /// Returns the part from its index text.
    pub fn parse(text: &str) -> Self {
        match text {
            "pacing" => Self::Pacing,
            _ => Self::Full,
        }
    }
}

/// The worst status of the validity checks of one measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CheckStatus {
    /// Every check passes.
    Pass,
    /// A check warns.
    Warn,
    /// A check fails, so the report counts as invalid.
    Fail,
}

impl CheckStatus {
    /// Returns the status of a check from its report text.
    pub fn parse(text: &str) -> Self {
        match text {
            "fail" => Self::Fail,
            "warn" => Self::Warn,
            _ => Self::Pass,
        }
    }

    /// Returns the lowercase name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

/// The validity of one measurement of a cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementHealth {
    /// The measurement, such as `hover-row` or `scroll-fling`.
    pub measurement: String,
    /// The report counts as valid.
    pub valid: bool,
    /// The worst check status.
    pub worst: CheckStatus,
}

/// One cell of a run: its key, where it lives and what it measured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellInfo {
    /// The index in the run.
    pub id: CellId,
    /// The matrix point.
    pub key: CellKey,
    /// The measurements the cell takes.
    pub part: Part,
    /// The cell directory relative to the run directory.
    pub dir: String,
    /// The measurements that wrote a report.
    pub done: Vec<String>,
    /// The measurements that failed, with their errors.
    pub errors: Vec<(String, String)>,
    /// The measurements the cell skips, with the reason.
    pub skipped: Vec<(String, String)>,
    /// A pacing measurement missed more than 1 % of the refreshes.
    pub missed_refreshes: bool,
    /// A response took over a second or never came, so the cell skipped its later measurements.
    pub over_budget: bool,
    /// Why the cell did not run: its framework was over the response budget at a smaller size.
    pub stopped: Option<String>,
    /// The validity of every bench report.
    pub health: Vec<MeasurementHealth>,
}

impl CellInfo {
    /// Returns the validity of a measurement, if the cell has its report.
    pub fn health_of(&self, measurement: &str) -> Option<&MeasurementHealth> {
        self.health.iter().find(|h| h.measurement == measurement)
    }

    /// Returns true when the measurement report exists and counts as valid.
    pub fn is_valid(&self, measurement: &str) -> bool {
        self.health_of(measurement).is_none_or(|h| h.valid)
    }

    /// Returns true when the cell ran.
    pub fn ran(&self) -> bool {
        self.stopped.is_none()
    }
}
