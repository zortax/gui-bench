//! The load of one run: the index, then every cell directory in parallel, then the metric
//! columns and the provenance.

use std::path::Path;

use explorer_core::api::{CellInfo, MeasurementHealth, RunOverview};
use rayon::prelude::*;

use super::RunData;
use super::columns::Columns;
use super::provenance;
use crate::catalog::{Entry, FORMAT};
use crate::error::{DataError, read_json};
use crate::report::{BenchReport, CellOutcome, RunIndex, Startup};

/// The files of one cell that the store keeps for a moment.
pub(super) struct LoadedCell {
    pub info: CellInfo,
    pub reports: Vec<(String, BenchReport)>,
    pub startup: Option<Startup>,
    pub churn: Option<serde_json::Value>,
}

/// Loads a run directory.
pub(super) fn load(entry: &Entry) -> Result<RunData, DataError> {
    let index: RunIndex = read_json(&entry.dir.join("run.json"))?;
    if index.format != Some(FORMAT) {
        return Err(DataError::Unsupported(
            entry.summary.id.clone(),
            format!("format {:?}", index.format),
        ));
    }
    let cells: Vec<LoadedCell> = index
        .cells
        .par_iter()
        .enumerate()
        .map(|(i, outcome)| load_cell(&entry.dir, i as u32, outcome))
        .collect();
    let mut columns = Columns::default();
    for cell in &cells {
        columns.add_cell(cell);
    }
    let provenance = provenance::collect(&index, entry, &cells);
    let overview = RunOverview {
        summary: entry.summary.clone(),
        provenance,
        cells: cells.into_iter().map(|c| c.info).collect(),
    };
    Ok(RunData {
        dir: entry.dir.clone(),
        overview,
        catalog: columns.catalog(),
        columns: columns.into_map(),
    })
}

fn load_cell(run_dir: &Path, id: u32, outcome: &CellOutcome) -> LoadedCell {
    let dir = run_dir.join(&outcome.dir);
    let mut reports = Vec::new();
    let mut startup = None;
    let mut churn = None;
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.strip_suffix(".json").map(str::to_owned)
        })
        .collect();
    names.sort();
    for name in names {
        let path = dir.join(format!("{name}.json"));
        match name.as_str() {
            "cell" => {}
            "startup" => startup = read_json(&path).ok(),
            "churn" => churn = read_json(&path).ok(),
            _ if name.starts_with("phases-") => {}
            _ => match read_json::<BenchReport>(&path) {
                Ok(report) if report.format_version.is_some() => reports.push((name, report)),
                Ok(_) => {}
                Err(e) => tracing::warn!("{e}"),
            },
        }
    }
    let health = reports
        .iter()
        .filter_map(|(name, report)| {
            let validity = report.validity.as_ref()?;
            let flagged: Vec<_> = validity
                .checks
                .iter()
                .filter(|c| c.status() != explorer_core::api::CheckStatus::Pass)
                .collect();
            Some(MeasurementHealth {
                measurement: name.clone(),
                valid: validity.valid,
                worst: flagged
                    .iter()
                    .map(|c| c.status())
                    .max()
                    .unwrap_or(explorer_core::api::CheckStatus::Pass),
            })
        })
        .collect();
    let info = CellInfo {
        id,
        key: outcome.cell.key(),
        dir: outcome.dir.clone(),
        done: outcome.done.clone(),
        errors: outcome.errors.clone(),
        skipped: outcome.skipped.clone(),
        health,
    };
    LoadedCell {
        info,
        reports,
        startup,
        churn,
    }
}
