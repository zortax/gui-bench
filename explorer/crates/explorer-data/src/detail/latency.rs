//! The latency series of a cell: every repeat per iteration, the stages, the latency per input
//! phase and the histogram.

use explorer_core::api::{
    CellId, HistogramBin, LatencyDetail, LatencyIteration, LatencySample, PhaseBin, Stage,
};
use serde_json::Value;

use super::json::{count, flag, iterations, num, pooled, refresh_ms, report, section, summary};
use crate::error::DataError;
use crate::store::RunData;

/// The stages from input to presentation, in order, with their report keys.
const STAGES: [(&str, &str); 4] = [
    ("input to commit", "input_to_commit_ms"),
    ("commit to ready", "commit_to_ready_ms"),
    ("ready to latch", "ready_to_latch_ms"),
    ("latch to present", "latch_to_present_ms"),
];

impl RunData {
    /// Returns the latency series of a measurement such as `hover` or `op-swap`.
    pub fn latency_detail(
        &self,
        id: CellId,
        measurement: &str,
    ) -> Result<LatencyDetail, DataError> {
        let report = report(&self.cell_dir(id)?, measurement)?;
        let latency = section(&report, "latency");
        let iterations = iterations(latency)
            .into_iter()
            .map(|(index, it)| LatencyIteration {
                index,
                samples: samples(it),
            })
            .collect();
        let pooled = pooled(latency);
        let get = |k: &str| pooled.and_then(|p| p.get(k));
        let stages = STAGES
            .iter()
            .map(|(name, key)| Stage {
                name: (*name).to_owned(),
                summary: summary(get(key)),
            })
            .collect();
        let by_phase = get("by_phase")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|b| PhaseBin {
                from_ms: num(b, "from_ms").unwrap_or(0.0),
                to_ms: num(b, "to_ms").unwrap_or(0.0),
                count: count(b, "count"),
                total_p50_ms: num(b, "total_p50_ms"),
            })
            .collect();
        let histogram = get("histogram")
            .and_then(|h| h.get("bins")?.as_array())
            .into_iter()
            .flatten()
            .map(|b| HistogramBin {
                lo: num(b, "lo").unwrap_or(0.0),
                hi: num(b, "hi").unwrap_or(0.0),
                count: count(b, "count"),
            })
            .collect();
        let median_ci = get("total_median_ci").and_then(|c| Some((num(c, "lo")?, num(c, "hi")?)));
        Ok(LatencyDetail {
            measurement: measurement.to_owned(),
            iterations,
            stages,
            total: summary(get("total_ms")),
            by_phase,
            histogram,
            median_ci,
            refresh_ms: refresh_ms(&report),
        })
    }
}

fn samples(report: &Value) -> Vec<LatencySample> {
    report
        .get("samples")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|s| LatencySample {
            repeat: count(s, "repeat"),
            phase_ms: num(s, "phase_ms").unwrap_or(0.0),
            total_ms: num(s, "total_ms"),
            input_to_commit_ms: num(s, "input_to_commit_ms"),
            timed_out: flag(s, "timed_out"),
        })
        .collect()
}
