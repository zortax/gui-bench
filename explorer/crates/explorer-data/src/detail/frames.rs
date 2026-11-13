//! The frame profiles of one matrix point: for every cell, the pooled interval percentiles and
//! the late frames of every iteration, placed on the time axis of the iteration. Scroll glides
//! list every frame; frame pacing lists the frames over its jank threshold.

use std::collections::HashMap;

use explorer_core::api::{CellId, FrameProfile, FrameQuantiles, GapFrame, GapLane};
use rayon::prelude::*;
use serde_json::Value;

use super::json::{count, flag, iterations, num, pooled, refresh_ms, report, section};
use crate::store::RunData;

/// A matrix point: the cells of every framework at one size and refresh rate.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub scenario: String,
    pub hz: f64,
    pub rows: u32,
}

fn records<'a>(v: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    v.get(key).and_then(Value::as_array).into_iter().flatten()
}

/// Returns the start and the length in ms of every iteration of a report, by index.
fn windows(report: &Value) -> HashMap<u32, (f64, f64)> {
    records(report, "iterations")
        .filter_map(|it| {
            let start = num(it, "started")?;
            let end = num(it, "finished")?;
            Some((count(it, "index"), (start, (end - start) / 1e6)))
        })
        .collect()
}

fn quantiles(v: Option<&Value>) -> Option<FrameQuantiles> {
    let v = v.filter(|v| !v.is_null())?;
    let f = |k| num(v, k).unwrap_or(0.0);
    let q = FrameQuantiles {
        count: count(v, "count"),
        mean: f("mean"),
        min: f("min"),
        p50: f("p50"),
        p90: f("p90"),
        p95: f("p95"),
        p99: f("p99"),
        p999: f("p999"),
        max: f("max"),
    };
    (q.count > 0).then_some(q)
}

/// Returns the lanes of a scroll glide: every step of the glide is a frame.
fn scroll_lanes(report: &Value, windows: &HashMap<u32, (f64, f64)>) -> Vec<GapLane> {
    iterations(section(report, "scroll_smoothness"))
        .into_iter()
        .filter_map(|(index, it)| {
            let (start, window_ms) = *windows.get(&index)?;
            let at = |s: &Value| num(s, "presented").map(|ns| (ns - start) / 1e6);
            let steps: Vec<&Value> = records(it, "steps").collect();
            let presents = steps
                .first()
                .and_then(|s| at(s))
                .zip(steps.last().and_then(|s| at(s)));
            let gaps = steps
                .iter()
                .filter(|s| {
                    count(s, "vblanks_missed") > 0 || flag(s, "stalled") || flag(s, "double_step")
                })
                .filter_map(|s| {
                    Some(GapFrame {
                        t_ms: at(s)?,
                        interval_ms: num(s, "interval_ms")?,
                        missed: count(s, "vblanks_missed"),
                        stalled: flag(s, "stalled"),
                        double_step: flag(s, "double_step"),
                    })
                })
                .collect();
            Some(GapLane {
                index,
                window_ms,
                presents,
                frames: steps.len() as u32,
                total_ms: steps.iter().filter_map(|s| num(s, "interval_ms")).sum(),
                complete: true,
                threshold_ms: 0.0,
                gaps,
            })
        })
        .collect()
}

/// Returns the lanes of frame pacing: the report lists the frames over 1.5 times the median
/// interval, which holds every late frame while the median is one refresh.
fn pacing_lanes(
    report: &Value,
    windows: &HashMap<u32, (f64, f64)>,
    refresh_ms: f64,
) -> Vec<GapLane> {
    iterations(section(report, "frame_pacing"))
        .into_iter()
        .filter_map(|(index, it)| {
            let (start, window_ms) = *windows.get(&index)?;
            let jank = it.get("jank");
            let threshold_ms = jank.and_then(|j| num(j, "threshold_ms")).unwrap_or(0.0);
            let gaps = jank
                .into_iter()
                .flat_map(|j| records(j, "frames"))
                .filter_map(|f| {
                    Some(GapFrame {
                        t_ms: (num(f, "presented")? - start) / 1e6,
                        interval_ms: num(f, "interval_ms")?,
                        missed: count(f, "vblanks_missed"),
                        stalled: false,
                        double_step: false,
                    })
                })
                .collect();
            Some(GapLane {
                index,
                window_ms,
                presents: None,
                frames: count(it, "presented"),
                total_ms: num(it, "duration_ms").unwrap_or(0.0),
                complete: threshold_ms <= 1.5 * refresh_ms + 0.01,
                threshold_ms,
                gaps,
            })
        })
        .collect()
}

fn profile(cell: CellId, report: &Value) -> FrameProfile {
    let refresh_ms = refresh_ms(report);
    let pacing = pooled(section(report, "frame_pacing"));
    let windows = windows(report);
    let mut lanes = scroll_lanes(report, &windows);
    if lanes.is_empty() {
        lanes = pacing_lanes(report, &windows, refresh_ms);
    }
    FrameProfile {
        cell,
        refresh_ms,
        quantiles: quantiles(pacing.and_then(|p| p.get("interval_ms"))),
        fps: pacing.and_then(|p| num(p, "fps")),
        missed_vblanks: pacing.map_or(0, |p| count(p, "missed_vblanks")),
        lanes,
    }
}

impl RunData {
    /// Returns the frame profiles of a measurement in every cell of a matrix point. Cells
    /// without the report give none.
    pub fn frame_profiles(&self, point: &Point, measurement: &str) -> Vec<FrameProfile> {
        let cells: Vec<(CellId, std::path::PathBuf)> = self
            .overview
            .cells
            .iter()
            .filter(|c| {
                c.key.scenario == point.scenario && c.key.hz == point.hz && c.key.rows == point.rows
            })
            .map(|c| (c.id, self.dir.join(&c.dir)))
            .collect();
        cells
            .par_iter()
            .filter_map(|(id, dir)| Some(profile(*id, &report(dir, measurement).ok()?)))
            .collect()
    }
}
