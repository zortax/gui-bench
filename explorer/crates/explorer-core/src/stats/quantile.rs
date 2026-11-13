//! Quantiles by the nearest-rank rule that the gui-bench plots use: the value at
//! `round((n - 1) * q)` of the sorted values, with ties rounded to the even index.

/// Returns the finite values in ascending order.
pub fn sorted(values: &[f64]) -> Vec<f64> {
    let mut out: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    out.sort_by(f64::total_cmp);
    out
}

/// Returns the quantile `q` (0 to 1) of sorted values, or `None` without values.
pub fn quantile(sorted: &[f64], q: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let last = sorted.len() - 1;
    let index = ((last as f64) * q.clamp(0.0, 1.0)).round_ties_even() as usize;
    Some(sorted[index.min(last)])
}

/// Returns the median of unsorted values.
pub fn median(values: &[f64]) -> Option<f64> {
    quantile(&sorted(values), 0.5)
}

/// Returns the first and third quartile of unsorted values.
pub fn iqr(values: &[f64]) -> Option<(f64, f64)> {
    let s = sorted(values);
    Some((quantile(&s, 0.25)?, quantile(&s, 0.75)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank() {
        let s = sorted(&[5.0, 1.0, 3.0, 2.0, 4.0]);
        assert_eq!(quantile(&s, 0.5), Some(3.0));
        assert_eq!(quantile(&s, 0.25), Some(2.0));
        assert_eq!(quantile(&s, 1.0), Some(5.0));
        assert_eq!(quantile(&[], 0.5), None);
        assert_eq!(quantile(&[1.0, 2.0], 0.5), Some(1.0));
        assert_eq!(quantile(&[1.0, 2.0, 3.0, 4.0], 0.5), Some(3.0));
    }

    #[test]
    fn skips_nan() {
        assert_eq!(median(&[f64::NAN, 2.0, 4.0, 3.0]), Some(3.0));
    }
}
