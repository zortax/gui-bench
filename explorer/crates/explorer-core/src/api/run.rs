//! Runs: the entry of a run in the catalog and the overview of one loaded run.

use serde::{Deserialize, Serialize};

use super::{CellInfo, Provenance};

/// Whether the explorer can read a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RunStatus {
    /// The run has format 2.
    Ok,
    /// The run has another format or a broken index.
    Unsupported(String),
}

/// A run in the catalog of the results directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    /// The directory name, which identifies the run.
    pub id: String,
    /// Links that point at the run, such as `latest`.
    pub aliases: Vec<String>,
    /// The start of the run, in Unix seconds.
    pub started_unix: Option<u64>,
    /// The last write of the run index, in Unix seconds.
    pub modified_unix: Option<u64>,
    /// The cells in the index.
    pub cells: u32,
    /// The cells with at least one failed measurement.
    pub failed_cells: u32,
    /// The scenarios the run selects.
    pub scenarios: Vec<String>,
    /// The frameworks the run selects.
    pub frameworks: Vec<String>,
    /// The run index changed a short time ago, so the runner probably still writes it.
    pub live: bool,
    /// Whether the explorer can read the run.
    pub status: RunStatus,
}

impl RunSummary {
    /// Returns true when the explorer can read the run.
    pub fn readable(&self) -> bool {
        self.status == RunStatus::Ok
    }
}

/// A loaded run: its provenance and its cells. The metric catalog travels on its own, since
/// few views need it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunOverview {
    /// The catalog entry.
    pub summary: RunSummary,
    /// Where the run comes from.
    pub provenance: Provenance,
    /// Every cell, indexed by its id.
    pub cells: Vec<CellInfo>,
}

impl RunOverview {
    /// Returns the frameworks of the cells, in no fixed order.
    pub fn frameworks(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for cell in &self.cells {
            if !out.contains(&cell.key.framework) {
                out.push(cell.key.framework.clone());
            }
        }
        out
    }
}
