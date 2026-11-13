//! The metrics that carry the story of the app scenario, and the cells of one framework that
//! ran at a refresh rate, smallest first.

use explorer_core::api::{CellInfo, MetricRef};
use explorer_core::story::{PACED, RESPONSES};

use super::build::Build;
use crate::state::Size;

/// The app scenario.
pub const APP: &str = "app";
/// The list scenario.
pub const LIST: &str = "list";

/// Returns the response time of a probe: every repeat, input to finished frame.
pub fn response(step: &str) -> MetricRef {
    MetricRef::new(step, "latency", "response_ms")
}

/// Returns the share of missed refreshes of a pacing measurement, per iteration, in percent.
pub fn missed(step: &str) -> MetricRef {
    MetricRef::new(step, "frame_pacing", "missed_share")
}

/// Returns the slowest frames of a pacing measurement: the 99th percentile of the interval.
pub fn slowest(step: &str) -> MetricRef {
    MetricRef::new(step, "frame_pacing", "interval_p99")
}

/// Returns the response metrics of the app scenario.
pub fn response_metrics() -> Vec<MetricRef> {
    RESPONSES.iter().map(|s| response(s.name)).collect()
}

/// Returns the missed-refresh metrics of the app scenario.
pub fn missed_metrics() -> Vec<MetricRef> {
    PACED.iter().map(|s| missed(s.name)).collect()
}

/// Returns the shown app cells of a framework at a refresh rate, smallest first.
pub fn app_series<'a>(b: &Build<'a>, framework: &str, hz: f64) -> Vec<&'a CellInfo> {
    b.cells(APP)
        .into_iter()
        .filter(|c| c.key.framework == framework && c.key.hz == hz)
        .collect()
}

/// Returns every size of a scenario in the run, the sizes that did not run included.
pub fn all_sizes(b: &Build, scenario: &str) -> Vec<Size> {
    let mut out: Vec<Size> = b
        .data
        .overview()
        .cells
        .iter()
        .filter(|c| c.key.scenario == scenario)
        .map(|c| (c.key.rows, c.key.elements))
        .collect();
    out.sort_by_key(|s| s.0);
    out.dedup();
    out
}

/// Breaks a title into lines of at most `width` characters at its spaces.
pub fn wrap(title: &str, width: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    for word in title.split(' ') {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines.join("\n")
}
