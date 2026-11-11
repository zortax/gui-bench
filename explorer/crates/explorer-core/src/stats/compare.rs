//! Comparison measures: ratios with intervals, Cliff's delta, the outlier fence of the focus mode
//! and the geometric mean of the scorecard.

use serde::{Deserialize, Serialize};

use super::quantile::sorted;

/// A ratio of two medians with an optional interval.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ratio {
    pub estimate: f64,
    pub lo: Option<f64>,
    pub hi: Option<f64>,
}

impl Ratio {
    /// Returns true when the interval holds 1, so the two groups do not differ clearly.
    pub fn spans_one(&self) -> bool {
        match (self.lo, self.hi) {
            (Some(lo), Some(hi)) => lo <= 1.0 && hi >= 1.0,
            _ => false,
        }
    }
}

/// Returns Cliff's delta of `a` against `b`: the share of pairs where `a` is larger minus the
/// share where it is smaller, from -1 to 1.
pub fn cliffs_delta(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.is_empty() || b.is_empty() {
        return None;
    }
    let mut more = 0i64;
    let mut less = 0i64;
    for x in a {
        for y in b {
            if x > y {
                more += 1;
            } else if x < y {
                less += 1;
            }
        }
    }
    Some((more - less) as f64 / (a.len() * b.len()) as f64)
}

/// Returns the upper bound of the focus range for a set of positive peak values, one per series
/// or row, or `None` when no group of values stands apart. The values split at the widest gap in
/// log space above the median, when that gap is at least a factor of two; the bound is the
/// largest value below the gap.
pub fn focus_limit(peaks: &[f64]) -> Option<f64> {
    let logs = sorted(
        &peaks
            .iter()
            .filter(|v| **v > 0.0)
            .map(|v| v.log10())
            .collect::<Vec<_>>(),
    );
    if logs.len() < 3 {
        return None;
    }
    let from = logs.len().div_ceil(2) - 1;
    let (cut, gap) = (from..logs.len() - 1)
        .map(|i| (i, logs[i + 1] - logs[i]))
        .max_by(|a, b| a.1.total_cmp(&b.1))?;
    (gap >= 2f64.log10()).then(|| 10f64.powf(logs[cut]))
}

/// Returns the geometric mean of positive values.
pub fn geomean(values: &[f64]) -> Option<f64> {
    let logs: Vec<f64> = values
        .iter()
        .filter(|v| **v > 0.0 && v.is_finite())
        .map(|v| v.ln())
        .collect();
    (!logs.is_empty()).then(|| (logs.iter().sum::<f64>() / logs.len() as f64).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_extremes() {
        assert_eq!(cliffs_delta(&[3.0, 4.0], &[1.0, 2.0]), Some(1.0));
        assert_eq!(cliffs_delta(&[1.0], &[1.0]), Some(0.0));
    }

    #[test]
    fn focus_drops_far_series() {
        let peaks = [14.0, 15.0, 13.0, 15.5, 59.0, 176.0, 75.0, 14.5];
        let limit = focus_limit(&peaks).unwrap();
        assert!((limit - 15.5).abs() < 1e-9);
        assert_eq!(focus_limit(&[10.0, 11.0, 12.0, 13.0]), None);
        assert_eq!(focus_limit(&[1.0, 1.1, 1.2, 1.3, 9.0]), Some(1.3));
    }

    #[test]
    fn geometric_mean() {
        assert!((geomean(&[1.0, 4.0]).unwrap() - 2.0).abs() < 1e-12);
    }
}
