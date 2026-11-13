//! The detail of one cell: its outcome, launches, and the checks, iterations and parameters of
//! every bench report; and the files a cell may show.

use std::path::PathBuf;

use explorer_core::api::{
    CellDetail, CellId, Check, CheckStatus, IterationInfo, MeasurementDetail,
};
use serde_json::Value;

use super::json::{count, flag, num};
use crate::error::{DataError, read_json};
use crate::report::{CellOutcome, Environment, Startup};
use crate::store::RunData;

/// The screenshots a cell may hold, by file stem: after startup, with the menu open, with the
/// live updates on, and of the messages page.
pub const SCREENSHOTS: [&str; 4] = [
    "screenshot",
    "screenshot-menu",
    "screenshot-live",
    "screenshot-messages",
];

/// A file of a cell that the explorer serves as is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellFile {
    /// A screenshot, by its place in [`SCREENSHOTS`].
    Screenshot(usize),
    Log,
}

impl CellFile {
    /// Returns the file of a name such as `screenshot-menu.png` or `app.log`.
    pub fn parse(name: &str) -> Option<Self> {
        if name == "app.log" {
            return Some(Self::Log);
        }
        let stem = name.strip_suffix(".png")?;
        SCREENSHOTS
            .iter()
            .position(|s| *s == stem)
            .map(Self::Screenshot)
    }

    fn name(self) -> String {
        match self {
            Self::Screenshot(i) => format!("{}.png", SCREENSHOTS[i]),
            Self::Log => "app.log".to_owned(),
        }
    }
}

impl RunData {
    /// Returns the path of a cell file, if it exists.
    pub fn cell_file(&self, id: CellId, file: CellFile) -> Result<PathBuf, DataError> {
        let path = self.cell_dir(id)?.join(file.name());
        path.is_file()
            .then_some(path)
            .ok_or_else(|| DataError::NoReport(file.name()))
    }

    /// Returns the detail of a cell.
    pub fn cell_detail(&self, id: CellId) -> Result<CellDetail, DataError> {
        let cell = self.cell(id)?.clone();
        let dir = self.cell_dir(id)?;
        let outcome: Option<CellOutcome> = read_json(&dir.join("cell.json")).ok();
        let startup: Startup = read_json(&dir.join("startup.json")).unwrap_or_default();
        let mut measurements = Vec::new();
        let mut environment = None;
        for name in cell.done.iter().chain(cell.errors.iter().map(|(n, _)| n)) {
            let Ok(report) = super::json::report(&dir, name) else {
                continue;
            };
            if report.get("format_version").is_none() {
                continue;
            }
            if environment.is_none() {
                environment = report
                    .get("environment")
                    .and_then(|e| serde_json::from_value::<Environment>(e.clone()).ok())
                    .map(|e| e.to_api());
            }
            measurements.push(measurement(name, &report));
        }
        Ok(CellDetail {
            args: outcome.map(|o| o.args).unwrap_or_default(),
            launches: startup.launches.iter().map(|l| l.to_api()).collect(),
            measurements,
            environment,
            screenshots: SCREENSHOTS
                .iter()
                .filter(|s| dir.join(format!("{s}.png")).is_file())
                .map(|s| (*s).to_owned())
                .collect(),
            has_log: dir.join("app.log").is_file(),
            flags: self
                .overview
                .provenance
                .builds
                .iter()
                .filter(|b| b.framework == cell.key.framework)
                .flat_map(|b| &b.flags)
                .map(|f| (f.part.clone(), f.note.clone()))
                .collect(),
            cell,
        })
    }
}

fn measurement(name: &str, report: &Value) -> MeasurementDetail {
    let validity = report.get("validity");
    let checks = validity
        .and_then(|v| v.get("checks")?.as_array())
        .into_iter()
        .flatten()
        .map(|c| Check {
            name: text(c, "name"),
            status: CheckStatus::parse(&text(c, "status")),
            message: text(c, "message"),
            value: num(c, "value"),
            limit: num(c, "limit"),
        })
        .collect();
    let iterations = report
        .get("iterations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|it| IterationInfo {
            index: count(it, "index"),
            measured: it.get("measured").and_then(Value::as_u64).map(|m| m as u32),
            ok: flag(it, "ok"),
            error: it.get("error").and_then(Value::as_str).map(str::to_owned),
            late_wakes: count(it, "late_wakes"),
            max_lateness_ms: num(it, "max_lateness").unwrap_or(0.0) / 1e6,
            duration_ms: (num(it, "finished").unwrap_or(0.0) - num(it, "started").unwrap_or(0.0))
                / 1e6,
        })
        .collect();
    let params = report
        .get("params")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, v)| (k.clone(), short(v)))
        .collect();
    let sections = report
        .get("sections")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|s| text(s, "name"))
        .collect();
    MeasurementDetail {
        name: name.to_owned(),
        preset: report
            .get("preset")
            .and_then(Value::as_str)
            .map(str::to_owned),
        valid: validity.is_some_and(|v| flag(v, "valid")),
        checks,
        iterations,
        params,
        sections,
    }
}

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn short(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(map) => map
            .iter()
            .map(|(k, v)| format!("{k} {}", short(v)))
            .collect::<Vec<_>>()
            .join(", "),
        other => other.to_string(),
    }
}
