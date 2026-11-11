//! Helpers that read fields of a parsed report.

use std::path::Path;

use explorer_core::api::Summary;
use serde_json::Value;

use crate::error::{DataError, read_json};

/// Reads a report of a cell directory by its file stem.
pub fn report(dir: &Path, name: &str) -> Result<Value, DataError> {
    let path = dir.join(format!("{name}.json"));
    if !path.is_file() {
        return Err(DataError::NoReport(name.to_owned()));
    }
    read_json(&path)
}

/// Returns a number field.
pub fn num(v: &Value, key: &str) -> Option<f64> {
    v.get(key)?.as_f64()
}

/// Returns a count field, zero when absent.
pub fn count(v: &Value, key: &str) -> u32 {
    v.get(key).and_then(Value::as_u64).unwrap_or(0) as u32
}

/// Returns a boolean field, false when absent.
pub fn flag(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// Returns a distribution summary, zeros when absent.
pub fn summary(v: Option<&Value>) -> Summary {
    let Some(v) = v else {
        return Summary::default();
    };
    let f = |k| num(v, k).unwrap_or(0.0);
    Summary {
        count: count(v, "count"),
        mean: f("mean"),
        stddev: f("stddev"),
        min: f("min"),
        max: f("max"),
        p50: f("p50"),
        p90: f("p90"),
        p95: f("p95"),
        p99: f("p99"),
    }
}

/// Returns a section of a report by name.
pub fn section<'a>(report: &'a Value, name: &str) -> Option<&'a Value> {
    report
        .get("sections")?
        .as_array()?
        .iter()
        .find(|s| s.get("name").and_then(Value::as_str) == Some(name))
}

/// Returns the measured iteration reports of a section with their index.
pub fn iterations(section: Option<&Value>) -> Vec<(u32, &Value)> {
    section
        .and_then(|s| s.get("iterations")?.as_array())
        .into_iter()
        .flatten()
        .filter(|it| it.get("variant").is_none_or(Value::is_null))
        .filter_map(|it| Some((count(it, "index"), it.get("report")?)))
        .collect()
}

/// Returns the pooled report of a section.
pub fn pooled(section: Option<&Value>) -> Option<&Value> {
    section?
        .get("pooled")?
        .as_array()?
        .iter()
        .find(|p| p.get("variant").is_none_or(Value::is_null))?
        .get("report")
}

/// Returns the refresh period of the first output in milliseconds.
pub fn refresh_ms(report: &Value) -> f64 {
    let hz = report
        .get("config")
        .and_then(|c| {
            c.get("outputs")?
                .as_array()?
                .first()?
                .get("mode")?
                .get("refresh")?
                .as_f64()
        })
        .unwrap_or(60.0);
    1000.0 / hz
}

/// Returns nanoseconds as milliseconds after a start.
pub fn ms_since(v: &Value, key: &str, start: f64) -> Option<f64> {
    num(v, key).map(|ns| (ns - start) / 1e6)
}
