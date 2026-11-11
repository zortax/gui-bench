//! Metrics: a reference by measurement, section and name, the catalog entry of a run, and the
//! per-cell statistics of one metric as a column.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::CellId;

/// A metric of a run: the measurement file, the report section and the metric name, such as
/// `hover.latency.total_p50`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MetricRef {
    /// The measurement, such as `hover`, `op-swap` or `startup`.
    pub measurement: String,
    /// The section of the report, such as `latency` or `resources`.
    pub section: String,
    /// The metric name in the section.
    pub metric: String,
}

impl MetricRef {
    /// Returns a reference from its parts.
    pub fn new(measurement: &str, section: &str, metric: &str) -> Self {
        Self {
            measurement: measurement.to_owned(),
            section: section.to_owned(),
            metric: metric.to_owned(),
        }
    }

    /// Parses `measurement.section.metric`. The measurement holds no dot.
    pub fn parse(text: &str) -> Option<Self> {
        let (measurement, rest) = text.split_once('.')?;
        let (section, metric) = rest.split_once('.')?;
        (!measurement.is_empty() && !section.is_empty() && !metric.is_empty())
            .then(|| Self::new(measurement, section, metric))
    }
}

impl fmt::Display for MetricRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.measurement, self.section, self.metric)
    }
}

/// The direction in which a metric improves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Better {
    /// Smaller values are better.
    Lower,
    /// Larger values are better.
    Higher,
    /// The metric has no direction.
    Neither,
}

impl Better {
    /// Returns the direction from its report text.
    pub fn parse(text: &str) -> Self {
        match text {
            "lower" => Self::Lower,
            "higher" => Self::Higher,
            _ => Self::Neither,
        }
    }
}

/// A metric that a run holds, with the scenarios that measure it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricInfo {
    /// The metric.
    pub metric: MetricRef,
    /// The unit, such as `ms`, `bytes` or `ms_per_s`.
    pub unit: String,
    /// The direction in which it improves.
    pub better: Better,
    /// The scenarios with at least one value.
    pub scenarios: Vec<String>,
    /// The number of cells with a value.
    pub cells: u32,
    /// The values of a statistic are per-repeat samples pooled over the iterations, and its CI
    /// is the CI of the median; otherwise they are one value per iteration.
    pub samples: bool,
}

/// The statistics of a metric in one cell, over the measured iterations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stat {
    /// The median iteration.
    pub median: f64,
    /// The mean of the iterations.
    pub mean: f64,
    /// The standard deviation of the iterations.
    pub stddev: f64,
    /// The smallest iteration.
    pub min: f64,
    /// The largest iteration.
    pub max: f64,
    /// The 95 % CI: of the mean over iterations, or of the median over samples.
    pub ci: Option<(f64, f64)>,
    /// One value per iteration, or every sample.
    pub values: Vec<f64>,
}

impl Stat {
    /// Returns a statistic of one value, such as a startup milestone.
    pub fn single(value: f64) -> Self {
        Self {
            median: value,
            mean: value,
            stddev: 0.0,
            min: value,
            max: value,
            ci: None,
            values: vec![value],
        }
    }

    /// Returns the statistic with every value multiplied by `factor`. A negative factor swaps the
    /// bounds.
    pub fn scaled(&self, factor: f64) -> Self {
        let (min, max) = order(self.min * factor, self.max * factor);
        Self {
            median: self.median * factor,
            mean: self.mean * factor,
            stddev: self.stddev * factor.abs(),
            min,
            max,
            ci: self.ci.map(|(lo, hi)| order(lo * factor, hi * factor)),
            values: self.values.iter().map(|v| v * factor).collect(),
        }
    }

    /// Returns the number of iterations.
    pub fn n(&self) -> usize {
        self.values.len()
    }

    /// Returns the statistic with every value rounded to five significant digits, which keeps
    /// every shown digit and halves the size on the wire.
    pub fn rounded(self) -> Self {
        let r = round5;
        Self {
            median: r(self.median),
            mean: r(self.mean),
            stddev: r(self.stddev),
            min: r(self.min),
            max: r(self.max),
            ci: self.ci.map(|(lo, hi)| (r(lo), r(hi))),
            values: self.values.into_iter().map(r).collect(),
        }
    }
}

/// Rounds a value to five significant digits.
pub fn round5(v: f64) -> f64 {
    if v == 0.0 || !v.is_finite() {
        return v;
    }
    let digits = 4 - v.abs().log10().floor() as i32;
    let p = 10f64.powi(digits);
    (v * p).round() / p
}

fn order(a: f64, b: f64) -> (f64, f64) {
    if a <= b { (a, b) } else { (b, a) }
}

/// One metric over the cells of a run: one statistic per cell that has it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricColumn {
    /// The metric.
    pub metric: MetricRef,
    /// The unit.
    pub unit: String,
    /// The direction in which it improves.
    pub better: Better,
    /// The values are per-repeat samples (see [`MetricInfo::samples`]).
    pub samples: bool,
    /// The cells with a value.
    pub cells: Vec<CellId>,
    /// The statistic of each cell, in the order of `cells`.
    pub stats: Vec<Stat>,
}

impl MetricColumn {
    /// Returns the statistic of a cell.
    pub fn of(&self, cell: CellId) -> Option<&Stat> {
        self.cells
            .iter()
            .position(|&c| c == cell)
            .map(|i| &self.stats[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_prints() {
        let r = MetricRef::parse("op-swap.latency.total_p50").unwrap();
        assert_eq!(r.measurement, "op-swap");
        assert_eq!(r.to_string(), "op-swap.latency.total_p50");
        assert!(MetricRef::parse("hover.latency").is_none());
    }

    #[test]
    fn rounds_to_five_digits() {
        assert_eq!(round5(13.1694645), 13.169);
        assert_eq!(round5(234_881_024.0), 234_880_000.0);
        assert_eq!(round5(0.000123456), 0.00012346);
    }

    #[test]
    fn scales_with_negative_factor() {
        let s = Stat {
            median: 2.0,
            mean: 2.0,
            stddev: 1.0,
            min: 1.0,
            max: 3.0,
            ci: Some((1.5, 2.5)),
            values: vec![1.0, 2.0, 3.0],
        };
        let t = s.scaled(-1.0);
        assert_eq!((t.min, t.max), (-3.0, -1.0));
        assert_eq!(t.ci, Some((-2.5, -1.5)));
    }
}
