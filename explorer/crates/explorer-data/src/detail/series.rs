//! The frame, resize and phase series of a cell.

use explorer_core::api::{
    CellId, ConfigureRecord, FrameIteration, FrameStep, FramesDetail, PacingIteration, PhaseSpan,
    PhasesDetail, ResizeDetail, ResizeIteration,
};
use serde_json::Value;

use super::json::{count, flag, iterations, ms_since, num, refresh_ms, report, section, summary};
use crate::error::{DataError, read_json};
use crate::store::RunData;

fn records<'a>(v: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    v.get(key).and_then(Value::as_array).into_iter().flatten()
}

impl RunData {
    /// Returns the frames of a measurement: scroll steps and frame pacing per iteration.
    pub fn frames_detail(&self, id: CellId, measurement: &str) -> Result<FramesDetail, DataError> {
        let report = report(&self.cell_dir(id)?, measurement)?;
        let scroll = iterations(section(&report, "scroll_smoothness"))
            .into_iter()
            .map(|(index, it)| {
                let start = records(it, "steps")
                    .find_map(|s| num(s, "presented"))
                    .unwrap_or(0.0);
                FrameIteration {
                    index,
                    steps: records(it, "steps")
                        .map(|s| FrameStep {
                            t_ms: ms_since(s, "presented", start).unwrap_or(0.0),
                            interval_ms: num(s, "interval_ms").unwrap_or(0.0),
                            expected_px: num(s, "expected_px").unwrap_or(0.0),
                            measured_px: num(s, "measured_px").unwrap_or(0.0),
                            stalled: flag(s, "stalled"),
                            double_step: flag(s, "double_step"),
                            vblanks_missed: count(s, "vblanks_missed"),
                        })
                        .collect(),
                    smoothness_score: num(it, "smoothness_score"),
                    start_latency_ms: num(it, "start_latency_ms"),
                }
            })
            .collect();
        let pacing = iterations(section(&report, "frame_pacing"))
            .into_iter()
            .map(|(index, it)| PacingIteration {
                index,
                fps: num(it, "fps").unwrap_or(0.0),
                interval: summary(it.get("interval_ms")),
                fps_series: records(it, "fps_series")
                    .map(|s| count(s, "frames"))
                    .collect(),
                jank_count: it.get("jank").map_or(0, |j| count(j, "count")),
                jank_threshold_ms: it
                    .get("jank")
                    .and_then(|j| num(j, "threshold_ms"))
                    .unwrap_or(0.0),
                missed_vblanks: count(it, "missed_vblanks"),
                discarded: count(it, "discarded"),
            })
            .collect();
        Ok(FramesDetail {
            measurement: measurement.to_owned(),
            scroll,
            pacing,
            refresh_ms: refresh_ms(&report),
        })
    }

    /// Returns the configures of a resize or scale measurement per iteration.
    pub fn resize_detail(&self, id: CellId, measurement: &str) -> Result<ResizeDetail, DataError> {
        let report = report(&self.cell_dir(id)?, measurement)?;
        let iterations = iterations(section(&report, "resize"))
            .into_iter()
            .map(|(index, it)| {
                let start = records(it, "records")
                    .find_map(|r| num(r, "sent"))
                    .unwrap_or(0.0);
                ResizeIteration {
                    index,
                    configures: records(it, "records")
                        .map(|r| {
                            let sent = num(r, "sent").unwrap_or(start);
                            ConfigureRecord {
                                t_ms: (sent - start) / 1e6,
                                width: count(r, "width"),
                                height: count(r, "height"),
                                ack_ms: ms_since(r, "acked", sent),
                                commit_ms: ms_since(r, "committed", sent),
                                present_ms: ms_since(r, "presented", sent),
                                superseded: flag(r, "superseded"),
                            }
                        })
                        .collect(),
                }
            })
            .collect();
        Ok(ResizeDetail {
            measurement: measurement.to_owned(),
            iterations,
            refresh_ms: refresh_ms(&report),
        })
    }

    /// Returns the app phases that zgui's marks record during a measurement.
    pub fn phases_detail(&self, id: CellId, measurement: &str) -> Result<PhasesDetail, DataError> {
        let path = self
            .cell_dir(id)?
            .join(format!("phases-{measurement}.json"));
        if !path.is_file() {
            return Err(DataError::NoReport(format!("phases-{measurement}")));
        }
        let value: Value = read_json(&path)?;
        let phases = value
            .as_object()
            .into_iter()
            .flatten()
            .map(|(name, p)| PhaseSpan {
                name: name.clone(),
                count: count(p, "count"),
                p50_ms: num(p, "p50_ms").unwrap_or(0.0),
                p95_ms: num(p, "p95_ms").unwrap_or(0.0),
                mean_ms: num(p, "mean_ms").unwrap_or(0.0),
                max_ms: num(p, "max_ms").unwrap_or(0.0),
                durations_ms: records(p, "durations_ms")
                    .filter_map(Value::as_f64)
                    .collect(),
            })
            .collect();
        Ok(PhasesDetail {
            measurement: measurement.to_owned(),
            phases,
        })
    }
}
