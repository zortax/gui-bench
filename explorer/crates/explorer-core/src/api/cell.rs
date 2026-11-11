//! A matrix cell: its key (scenario, framework, variant, refresh rate, size), its outcome and the
//! health of its measurements.

use serde::{Deserialize, Serialize};

/// The index of a cell in its run, stable for one load of the run.
pub type CellId = u32;

/// One point of the benchmark matrix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellKey {
    /// The scenario, such as `rows` or `table`.
    pub scenario: String,
    /// The framework, such as `zgui` or `gpui-kit-fast`.
    pub framework: String,
    /// The variant, `default` for scenarios without variants.
    pub variant: String,
    /// The refresh rate of the output in Hz.
    pub hz: f64,
    /// The number of rows the app renders.
    pub rows: u32,
    /// The ticker row updates per tick.
    pub updates: Option<u32>,
}

impl CellKey {
    /// Returns the framework, with the variant when it has one.
    pub fn label(&self) -> String {
        if self.variant == "default" {
            self.framework.clone()
        } else {
            format!("{} {}", self.framework, self.variant)
        }
    }

    /// Returns the size as text: the row count, with the ticker updates.
    pub fn size(&self) -> String {
        let rows = format!("{} rows", crate::units::thousands(f64::from(self.rows)));
        match self.updates {
            Some(u) => format!("{rows}, {u} updates/tick"),
            None => rows,
        }
    }

    /// Returns the size as a sortable pair: rows, then updates.
    pub fn size_key(&self) -> (u32, u32) {
        (self.rows, self.updates.unwrap_or(0))
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
    /// The measurement, such as `hover` or `op-swap`.
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
    /// The cell directory relative to the run directory.
    pub dir: String,
    /// The measurements that wrote a report.
    pub done: Vec<String>,
    /// The measurements that failed, with their errors.
    pub errors: Vec<(String, String)>,
    /// The measurements the cell skips, with the reason.
    pub skipped: Vec<(String, String)>,
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
}
