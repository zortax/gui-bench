//! The metric columns of a run: every metric summary of every report, and derived metrics: the
//! pooled latency and response time of every repeat, the startup milestones of every launch, and
//! the frame interval percentiles and the share of missed refreshes of every iteration.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use explorer_core::api::{Better, CellId, MetricColumn, MetricInfo, MetricRef, Stat};
use explorer_core::stats::{median_ci, quantile, sorted};

use super::load::LoadedCell;
use crate::report::{QuantilesRaw, Section};

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
                for (metric, stat) in latency_metrics(section) {
                    let r = MetricRef::new(name, &section.name, metric);
                    self.push(r, ("ms", Better::Lower, true), id, &scenario, stat);
                }
                if section.name == "frame_pacing" {
                    for (metric, unit, stat) in pacing_metrics(section) {
                        let r = MetricRef::new(name, &section.name, metric);
                        self.push(r, (unit, Better::Lower, false), id, &scenario, stat);
                    }
                }
            }
        }
        for metric in [
            "connected_ms",
            "first_surface_ms",
            "toplevel_ms",
            "first_buffer_ms",
            "first_present_ms",
            "content_ms",
        ] {
            let values: Vec<f64> = cell
                .launches
                .iter()
                .filter_map(|l| l.milestones().into_iter().find(|(m, _)| *m == metric)?.1)
                .collect();
            if let Some(stat) = stat_of(values) {
                let r = MetricRef::new("startup", "startup", metric);
                self.push(r, ("ms", Better::Lower, false), id, &scenario, stat);
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

/// Returns the per-repeat metrics of a latency section, pooled over the iterations: the time to
/// the photon (`total_ms`) with the report's CI of the median, and the response time
/// (`response_ms`): the time from the input to the commit of every repeat plus the median time
/// until the GPU finished the frame. Repeats that timed out drop out.
fn latency_metrics(section: &Section) -> Vec<(&'static str, Stat)> {
    let Some(report) = section
        .pooled
        .iter()
        .find(|p| p.variant.is_none())
        .and_then(|p| p.report.as_ref())
    else {
        return Vec::new();
    };
    let samples = || report.samples.iter().filter(|s| !s.timed_out);
    let ready = section
        .metrics
        .iter()
        .find(|m| m.name == "commit_to_ready_p50" && m.variant.is_none())
        .and_then(|m| m.median)
        .unwrap_or(0.0);
    let total = stat_of(samples().filter_map(|s| s.total_ms).collect()).map(|mut s| {
        s.ci = report.total_median_ci.as_ref().map(|c| (c.lo, c.hi));
        s
    });
    let response = stat_of(
        samples()
            .filter_map(|s| s.input_to_commit_ms)
            .map(|v| v + ready)
            .collect(),
    )
    .map(|mut s| {
        s.ci = median_ci(&s.values, 0.95);
        s
    });
    [("total_ms", total), ("response_ms", response)]
        .into_iter()
        .filter_map(|(m, s)| Some((m, s?)))
        .collect()
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
/// and p99.9 of every iteration, and the share of its refreshes that showed no new frame (the
/// missed refreshes over the missed and the presented ones), in percent.
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
    let missed: Vec<f64> = reports
        .iter()
        .filter_map(|r| {
            let (missed, shown) = (r.missed_vblanks?, r.presented?);
            (missed + shown > 0.0).then(|| 100.0 * missed / (missed + shown))
        })
        .collect();
    [
        ("interval_p90", "ms", quantile(|q| q.p90)),
        ("interval_p95", "ms", quantile(|q| q.p95)),
        ("interval_p999", "ms", quantile(|q| q.p999)),
        ("missed_share", "percent", missed),
    ]
    .into_iter()
    .filter_map(|(metric, unit, values)| Some((metric, unit, stat_of(values)?)))
    .collect()
}
