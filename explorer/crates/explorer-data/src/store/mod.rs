//! The run store: the catalog of the results directory and a small cache of loaded runs. A load
//! parses every report of a run once; callers that ask for the same run during a load wait for
//! that load. Every call blocks, so async callers run it on a blocking thread.

mod columns;
mod load;
mod provenance;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

use explorer_core::api::{
    CellId, CellInfo, MetricColumn, MetricInfo, MetricRef, RunOverview, RunSummary,
};

use crate::catalog;
use crate::error::DataError;

/// The loaded runs the cache keeps.
const CACHED_RUNS: usize = 4;

/// A loaded run: its overview and the column of every metric.
#[derive(Debug)]
pub struct RunData {
    /// The run directory.
    pub dir: PathBuf,
    pub overview: RunOverview,
    /// Every metric with at least one value.
    pub catalog: Vec<MetricInfo>,
    columns: std::collections::HashMap<MetricRef, MetricColumn>,
}

impl RunData {
    /// Returns the columns of the metrics, in the order asked. A metric the run lacks gives an
    /// empty column.
    pub fn columns(&self, metrics: &[MetricRef]) -> Vec<MetricColumn> {
        metrics
            .iter()
            .map(|m| {
                self.columns
                    .get(m)
                    .cloned()
                    .unwrap_or_else(|| MetricColumn {
                        metric: m.clone(),
                        unit: String::new(),
                        better: explorer_core::api::Better::Neither,
                        samples: false,
                        cells: Vec::new(),
                        stats: Vec::new(),
                    })
            })
            .collect()
    }

    /// Returns the columns of the metrics with the cells of some frameworks only. No frameworks
    /// keeps every cell.
    pub fn columns_of(&self, metrics: &[MetricRef], frameworks: &[String]) -> Vec<MetricColumn> {
        let mut columns = self.columns(metrics);
        if frameworks.is_empty() {
            return columns;
        }
        for column in &mut columns {
            let keep: Vec<bool> = column
                .cells
                .iter()
                .map(|id| {
                    self.overview
                        .cells
                        .get(*id as usize)
                        .is_some_and(|c| frameworks.contains(&c.key.framework))
                })
                .collect();
            let mut i = 0;
            column.cells.retain(|_| {
                i += 1;
                keep[i - 1]
            });
            let mut j = 0;
            column.stats.retain(|_| {
                j += 1;
                keep[j - 1]
            });
        }
        columns
    }

    /// Returns a cell by id.
    pub fn cell(&self, id: CellId) -> Result<&CellInfo, DataError> {
        self.overview
            .cells
            .get(id as usize)
            .ok_or(DataError::NoCell(id))
    }

    /// Returns the directory of a cell.
    pub fn cell_dir(&self, id: CellId) -> Result<PathBuf, DataError> {
        Ok(self.dir.join(&self.cell(id)?.dir))
    }
}

type Slot = Arc<OnceLock<Result<Arc<RunData>, String>>>;

struct Cached {
    id: String,
    modified: SystemTime,
    slot: Slot,
    used: u64,
}

/// The runs of a results directory.
pub struct Store {
    results: PathBuf,
    cache: Mutex<(u64, Vec<Cached>)>,
}

impl Store {
    /// Returns a store over a results directory.
    pub fn new(results: impl Into<PathBuf>) -> Self {
        Self {
            results: results.into(),
            cache: Mutex::new((0, Vec::new())),
        }
    }

    /// Returns the results directory.
    pub fn results(&self) -> &Path {
        &self.results
    }

    /// Returns every run, newest first.
    pub fn runs(&self) -> Vec<RunSummary> {
        catalog::scan(&self.results)
            .into_iter()
            .map(|e| e.summary)
            .collect()
    }

    /// Returns a loaded run by its id or an alias such as `latest`. A run whose index changed
    /// since its load loads again.
    pub fn run(&self, id: &str) -> Result<Arc<RunData>, DataError> {
        let entry = catalog::scan(&self.results)
            .into_iter()
            .find(|e| e.summary.id == id || e.summary.aliases.iter().any(|a| a == id))
            .ok_or_else(|| DataError::NoRun(id.to_owned()))?;
        if let explorer_core::api::RunStatus::Unsupported(reason) = &entry.summary.status {
            return Err(DataError::Unsupported(
                entry.summary.id.clone(),
                reason.clone(),
            ));
        }
        let slot = self.slot(&entry);
        let result = slot.get_or_init(|| {
            let started = std::time::Instant::now();
            let loaded = load::load(&entry).map(Arc::new).map_err(|e| e.to_string());
            tracing::info!(run = %entry.summary.id, ms = started.elapsed().as_millis(), "loaded run");
            loaded
        });
        match result {
            Ok(data) => Ok(data.clone()),
            Err(message) => {
                self.forget(&entry.summary.id);
                Err(DataError::Load(message.clone()))
            }
        }
    }

    fn slot(&self, entry: &catalog::Entry) -> Slot {
        let mut guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        let (clock, cached) = &mut *guard;
        *clock += 1;
        let now = *clock;
        cached.retain(|c| c.id != entry.summary.id || c.modified == entry.modified);
        if let Some(c) = cached.iter_mut().find(|c| c.id == entry.summary.id) {
            c.used = now;
            return c.slot.clone();
        }
        if cached.len() >= CACHED_RUNS
            && let Some(oldest) = cached
                .iter()
                .enumerate()
                .min_by_key(|(_, c)| c.used)
                .map(|(i, _)| i)
        {
            cached.remove(oldest);
        }
        let slot = Slot::default();
        cached.push(Cached {
            id: entry.summary.id.clone(),
            modified: entry.modified,
            slot: slot.clone(),
            used: now,
        });
        slot
    }

    fn forget(&self, id: &str) {
        let mut guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        guard.1.retain(|c| c.id != id);
    }
}
