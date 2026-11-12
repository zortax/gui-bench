//! The metric columns of a run: every metric summary of every report, the pooled latency
//! samples, and derived metrics: CPU per table operation, memory kept per churn cycle, startup
//! milestones, and the frame interval percentiles and stale refreshes of every iteration.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use explorer_core::api::{Better, CellId, MetricColumn, MetricInfo, MetricRef, Stat};
use explorer_core::stats::{quantile, sorted};

use super::load::LoadedCell;
use crate::report::{BenchReport, QuantilesRaw, Section};

#[derive(Default)]
pub(super) struct Columns {
    map: BTreeMap<MetricRef, (MetricColumn, BTreeSet<String>)>,
}

impl Columns {
    fn push(
        &mut self,
        metric: MetricRef,
        meta: (&str, Better, bool),
        cell: CellId,
        scenario: &str,
        stat: Stat,
    ) {
        let (unit, better, samples) = meta;
        let (column, scenarios) = self.map.entry(metric.clone()).or_insert_with(|| {
            (
                MetricColumn {
                    metric,
                    unit: unit.to_owned(),
                    better,
                    samples,
                    cells: Vec::new(),
                    stats: Vec::new(),
                },
                BTreeSet::new(),
            )
        });
        column.cells.push(cell);
        column.stats.push(stat.rounded());
        scenarios.insert(scenario.to_owned());
    }

    /// Adds the metrics of one cell.
    pub fn add_cell(&mut self, cell: &LoadedCell) {
        let id = cell.info.id;
        let scenario = cell.info.key.scenario.clone();
        for (name, report) in &cell.reports {
            for section in &report.sections {
                for m in section.metrics.iter().filter(|m| m.variant.is_none()) {
                    if let Some(stat) = m.stat() {
                        let r = MetricRef::new(name, &section.name, &m.name);
                        self.push(r, (&m.unit, m.better(), false), id, &scenario, stat);
                    }
                }
                if let Some(stat) = pooled_latency(section) {
                    let r = MetricRef::new(name, &section.name, "total_ms");
                    self.push(r, ("ms", Better::Lower, true), id, &scenario, stat);
                }
                if section.name == "frame_pacing" {
                    for (metric, unit, stat) in pacing_metrics(section) {
                        let r = MetricRef::new(name, &section.name, metric);
                        self.push(r, (unit, Better::Lower, false), id, &scenario, stat);
                    }
                }
            }
            if name.starts_with("op-")
                && let Some(stat) = cpu_per_op(report)
            {
                let r = MetricRef::new(name, "resources", "cpu_ms_per_op");
                self.push(r, ("ms_per_op", Better::Lower, false), id, &scenario, stat);
            }
        }
        if let Some(startup) = &cell.startup {
            for (metric, value) in [
                ("connected_ms", startup.connected_ms),
                ("first_surface_ms", startup.first_surface_ms),
                ("toplevel_ms", startup.toplevel_ms),
                ("first_buffer_ms", startup.first_buffer_ms),
                ("first_present_ms", startup.first_present_ms),
            ] {
                if let Some(v) = value {
                    let r = MetricRef::new("startup", "startup", metric);
                    self.push(
                        r,
                        ("ms", Better::Lower, false),
                        id,
                        &scenario,
                        Stat::single(v),
                    );
                }
            }
        }
        if let Some(churn) = &cell.churn {
            for (metric, keys) in [
                ("pss_kept_per_cycle", &["pss_bytes", "rss_bytes"][..]),
                ("private_kept_per_cycle", &["private_bytes"][..]),
                ("vram_kept_per_cycle", &["gpu_mem_bytes"][..]),
            ] {
                if let Some(v) = kept_per_cycle(churn, keys) {
                    let r = MetricRef::new("churn", "memory", metric);
                    self.push(
                        r,
                        ("kib", Better::Lower, false),
                        id,
                        &scenario,
                        Stat::single(v),
                    );
                }
            }
        }
    }

    /// Returns the catalog entry of every metric.
    pub fn catalog(&self) -> Vec<MetricInfo> {
        self.map
            .values()
            .map(|(c, scenarios)| MetricInfo {
                metric: c.metric.clone(),
                unit: c.unit.clone(),
                better: c.better,
                scenarios: scenarios.iter().cloned().collect(),
                cells: c.cells.len() as u32,
                samples: c.samples,
            })
            .collect()
    }

    pub fn into_map(self) -> HashMap<MetricRef, MetricColumn> {
        self.map.into_iter().map(|(k, (c, _))| (k, c)).collect()
    }
}

/// Returns the pooled per-repeat latency of a latency section: every sample that did not time
/// out, its median and the report's CI of the median.
fn pooled_latency(section: &Section) -> Option<Stat> {
    let pooled = section.pooled.iter().find(|p| p.variant.is_none())?;
    let report = pooled.report.as_ref()?;
    let values: Vec<f64> = report
        .samples
        .iter()
        .filter(|s| !s.timed_out)
        .filter_map(|s| s.total_ms)
        .collect();
    let s = sorted(&values);
    let median = quantile(&s, 0.5)?;
    let n = s.len() as f64;
    let mean = s.iter().sum::<f64>() / n;
    let stddev = (s.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n.max(1.0)).sqrt();
    Some(Stat {
        median,
        mean,
        stddev,
        min: s[0],
        max: s[s.len() - 1],
        ci: report.total_median_ci.as_ref().map(|c| (c.lo, c.hi)),
        values,
    })
}

/// Returns the statistic of per-iteration values, or `None` without any.
fn stat_of(values: Vec<f64>) -> Option<Stat> {
    let s = sorted(&values);
    let median = quantile(&s, 0.5)?;
    let n = s.len() as f64;
    let mean = s.iter().sum::<f64>() / n;
    let stddev = (s.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();
    Some(Stat {
        median,
        mean,
        stddev,
        min: s[0],
        max: s[s.len() - 1],
        ci: None,
        values,
    })
}

/// Returns the frame metrics that the report summaries lack: the interval percentiles p90, p95
/// and p99.9 of every iteration, and the share of refreshes that repeat the last frame.
fn pacing_metrics(section: &Section) -> Vec<(&'static str, &'static str, Stat)> {
    let reports: Vec<_> = section
        .iterations
        .iter()
        .filter(|it| it.variant.is_none())
        .filter_map(|it| it.report.as_ref())
        .collect();
    let quantile = |pick: fn(&QuantilesRaw) -> Option<f64>| {
        reports
            .iter()
            .filter_map(|r| r.interval_ms.as_ref().and_then(pick))
            .collect::<Vec<_>>()
    };
    let stale: Vec<f64> = reports
        .iter()
        .filter_map(|r| {
            let refreshes = r.duration_ms? * r.refresh_hz? / 1000.0;
            let missed = r.missed_vblanks?;
            (refreshes >= 1.0).then(|| (100.0 * missed / refreshes).min(100.0))
        })
        .collect();
    [
        ("interval_p90", "ms", quantile(|q| q.p90)),
        ("interval_p95", "ms", quantile(|q| q.p95)),
        ("interval_p999", "ms", quantile(|q| q.p999)),
        ("stale_refreshes", "percent", stale),
    ]
    .into_iter()
    .filter_map(|(metric, unit, values)| Some((metric, unit, stat_of(values)?)))
    .collect()
}

/// Returns the app CPU ms per table operation per iteration: every probe repeat runs the
/// operation and its undo, so an iteration holds two operations per repeat.
fn cpu_per_op(report: &BenchReport) -> Option<Stat> {
    let cpu = report
        .section("resources")?
        .metrics
        .iter()
        .find(|m| m.name == "cpu_ms" && m.variant.is_none())?
        .stat()?;
    let repeats = report.repeats().unwrap_or(1.0).max(1.0);
    Some(cpu.scaled(1.0 / (2.0 * repeats)))
}

/// Returns the KiB per replace cycle that the app keeps: memory after minus before the churn,
/// over the cycles. The first key with both values counts.
fn kept_per_cycle(churn: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    let cycles = churn.get("cycles")?.as_f64()?.max(1.0);
    let before = churn.get("before")?.get("process")?;
    let after = churn.get("after")?.get("process")?;
    keys.iter().find_map(|k| {
        let b = before.get(*k)?.as_f64()?;
        let a = after.get(*k)?.as_f64()?;
        Some((a - b) / cycles / 1024.0)
    })
}
