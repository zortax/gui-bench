//! The catalog of a results directory: every directory with a `run.json`, links such as `latest`
//! folded into their target, newest first.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use explorer_core::api::{RunStatus, RunSummary};
use serde::Deserialize;

use crate::error::read_json;

/// A run index that changed within this time counts as live.
const LIVE: Duration = Duration::from_secs(180);
/// The run format the explorer reads.
pub const FORMAT: u32 = 3;

/// A run in the catalog, with its directory and the last write of its index.
#[derive(Debug, Clone)]
pub struct Entry {
    pub summary: RunSummary,
    pub dir: PathBuf,
    pub modified: SystemTime,
}

/// The header of a run index: the fields the catalog shows.
#[derive(Debug, Deserialize)]
struct Header {
    format: Option<u32>,
    started_unix: Option<u64>,
    #[serde(default)]
    scenarios: Vec<String>,
    #[serde(default)]
    frameworks: Vec<String>,
    #[serde(default)]
    cells: Vec<HeaderCell>,
}

#[derive(Debug, Deserialize)]
struct HeaderCell {
    #[serde(default)]
    errors: Vec<serde_json::Value>,
    cell: Option<HeaderKey>,
}

#[derive(Debug, Deserialize)]
struct HeaderKey {
    scenario: Option<String>,
    framework: Option<String>,
}

/// Returns every run of a results directory, newest first.
pub fn scan(results: &Path) -> Vec<Entry> {
    let Ok(dir) = std::fs::read_dir(results) else {
        return Vec::new();
    };
    let mut by_target: BTreeMap<PathBuf, (Vec<String>, Option<String>)> = BTreeMap::new();
    for item in dir.flatten() {
        let path = item.path();
        if !path.join("run.json").is_file() {
            continue;
        }
        let name = item.file_name().to_string_lossy().into_owned();
        let target = path.canonicalize().unwrap_or(path.clone());
        let is_link = item.file_type().is_ok_and(|t| t.is_symlink());
        let slot = by_target.entry(target).or_default();
        if is_link {
            slot.0.push(name);
        } else {
            slot.1 = Some(name);
        }
    }
    let mut out: Vec<Entry> = by_target
        .into_iter()
        .map(|(target, (mut aliases, real))| {
            aliases.sort();
            let id = real.unwrap_or_else(|| aliases.remove(0));
            entry(id, aliases, target)
        })
        .collect();
    out.sort_by(|a, b| {
        b.summary
            .started_unix
            .cmp(&a.summary.started_unix)
            .then_with(|| a.summary.id.cmp(&b.summary.id))
    });
    out
}

fn entry(id: String, aliases: Vec<String>, dir: PathBuf) -> Entry {
    let modified = std::fs::metadata(dir.join("run.json"))
        .and_then(|m| m.modified())
        .unwrap_or(UNIX_EPOCH);
    let live = SystemTime::now()
        .duration_since(modified)
        .is_ok_and(|age| age < LIVE);
    let modified_unix = modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs());
    let (status, header) = match read_json::<Header>(&dir.join("run.json")) {
        Ok(h) if h.format == Some(FORMAT) => (RunStatus::Ok, Some(h)),
        Ok(h) => {
            let format = h.format.map_or("1".to_owned(), |f| f.to_string());
            let reason = format!("format {format}; the explorer reads format {FORMAT}");
            (RunStatus::Unsupported(reason), Some(h))
        }
        Err(e) => (RunStatus::Unsupported(e.to_string()), None),
    };
    let header = header.unwrap_or(Header {
        format: None,
        started_unix: None,
        scenarios: Vec::new(),
        frameworks: Vec::new(),
        cells: Vec::new(),
    });
    let scenarios = present(
        &header.scenarios,
        header
            .cells
            .iter()
            .filter_map(|c| c.cell.as_ref()?.scenario.as_deref()),
    );
    let frameworks = present(
        &header.frameworks,
        header
            .cells
            .iter()
            .filter_map(|c| c.cell.as_ref()?.framework.as_deref()),
    );
    let summary = RunSummary {
        id,
        aliases,
        started_unix: header.started_unix,
        modified_unix,
        cells: header.cells.len() as u32,
        failed_cells: header.cells.iter().filter(|c| !c.errors.is_empty()).count() as u32,
        scenarios,
        frameworks,
        live,
        status,
    };
    Entry {
        summary,
        dir,
        modified,
    }
}

/// Returns the names that cells use, in the order of the selection, then any others. Without
/// cells it returns the selection.
fn present<'a>(selected: &[String], used: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for name in used {
        if !seen.iter().any(|s| s == name) {
            seen.push(name.to_owned());
        }
    }
    if seen.is_empty() {
        return selected.to_vec();
    }
    let mut out: Vec<String> = selected
        .iter()
        .filter(|s| seen.contains(s))
        .cloned()
        .collect();
    out.extend(seen.into_iter().filter(|s| !selected.contains(s)));
    out
}
