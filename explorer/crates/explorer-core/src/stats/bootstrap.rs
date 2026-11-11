//! Seeded bootstrap intervals. The generator is deterministic, so an interval stays the same on
//! every render and in every export.

use super::quantile::{quantile, sorted};

/// The resamples of every bootstrap.
const RESAMPLES: usize = 2000;
/// The seed of every bootstrap.
const SEED: u64 = 0x9e37_79b9_7f4a_7c15;

/// A small xorshift64* generator.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// Returns a generator with a seed. A zero seed takes one.
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    /// Returns the next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// Returns an index below `n`.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

fn resample_median(values: &[f64], rng: &mut Rng, buf: &mut Vec<f64>) -> f64 {
    buf.clear();
    buf.extend((0..values.len()).map(|_| values[rng.below(values.len())]));
    buf.sort_by(f64::total_cmp);
    quantile(buf, 0.5).unwrap_or(f64::NAN)
}

/// Returns the percentile bootstrap interval of the median at a confidence such as 0.95. It
/// needs at least three values.
pub fn median_ci(values: &[f64], confidence: f64) -> Option<(f64, f64)> {
    let values = sorted(values);
    if values.len() < 3 {
        return None;
    }
    let mut rng = Rng::new(SEED);
    let mut buf = Vec::with_capacity(values.len());
    let mut medians: Vec<f64> = (0..RESAMPLES)
        .map(|_| resample_median(&values, &mut rng, &mut buf))
        .collect();
    medians.sort_by(f64::total_cmp);
    let tail = (1.0 - confidence) / 2.0;
    Some((quantile(&medians, tail)?, quantile(&medians, 1.0 - tail)?))
}

/// Returns the ratio of the medians `a / b` with its percentile bootstrap interval, resampling
/// both groups. Both groups need positive medians.
pub fn ratio_ci(a: &[f64], b: &[f64], confidence: f64) -> Option<super::Ratio> {
    let a = sorted(a);
    let b = sorted(b);
    let estimate = quantile(&a, 0.5)? / quantile(&b, 0.5)?;
    if !estimate.is_finite() || estimate <= 0.0 {
        return None;
    }
    if a.len() < 2 || b.len() < 2 {
        return Some(super::Ratio {
            estimate,
            lo: None,
            hi: None,
        });
    }
    let mut rng = Rng::new(SEED);
    let mut buf = Vec::new();
    let mut ratios: Vec<f64> = (0..RESAMPLES)
        .map(|_| resample_median(&a, &mut rng, &mut buf) / resample_median(&b, &mut rng, &mut buf))
        .filter(|r| r.is_finite())
        .collect();
    ratios.sort_by(f64::total_cmp);
    let tail = (1.0 - confidence) / 2.0;
    Some(super::Ratio {
        estimate,
        lo: quantile(&ratios, tail),
        hi: quantile(&ratios, 1.0 - tail),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_covers_median() {
        let v: Vec<f64> = (1..=40).map(f64::from).collect();
        let a = median_ci(&v, 0.95).unwrap();
        let b = median_ci(&v, 0.95).unwrap();
        assert_eq!(a, b);
        assert!(a.0 <= 20.0 && a.1 >= 21.0 && a.0 > 10.0 && a.1 < 31.0);
    }

    #[test]
    fn ratio_of_scaled_groups() {
        let a: Vec<f64> = (1..=30).map(|i| f64::from(i) * 2.0).collect();
        let b: Vec<f64> = (1..=30).map(f64::from).collect();
        let r = ratio_ci(&a, &b, 0.95).unwrap();
        assert!((r.estimate - 2.0).abs() < 0.1);
        assert!(r.lo.unwrap() < 2.0 && r.hi.unwrap() > 2.0);
    }
}
