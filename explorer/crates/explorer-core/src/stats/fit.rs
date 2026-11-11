//! Least-squares line fits, such as the memory that one more row costs.

/// A fitted line `y = intercept + slope * x` with the 95 % interval of the slope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub slope: f64,
    pub intercept: f64,
    /// The 95 % interval of the slope; `None` with fewer than three points.
    pub slope_ci: Option<(f64, f64)>,
}

/// The two-sided 97.5 % quantile of Student's t for 1 to 10 degrees of freedom.
const T975: [f64; 10] = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228,
];

/// Returns the least-squares line through the points, or `None` with fewer than two distinct x.
pub fn linear_fit(xs: &[f64], ys: &[f64]) -> Option<Fit> {
    let n = xs.len().min(ys.len());
    if n < 2 {
        return None;
    }
    let nf = n as f64;
    let mx = xs[..n].iter().sum::<f64>() / nf;
    let my = ys[..n].iter().sum::<f64>() / nf;
    let sxx: f64 = xs[..n].iter().map(|x| (x - mx).powi(2)).sum();
    if sxx <= 0.0 {
        return None;
    }
    let sxy: f64 = xs[..n]
        .iter()
        .zip(&ys[..n])
        .map(|(x, y)| (x - mx) * (y - my))
        .sum();
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let slope_ci = (n > 2).then(|| {
        let sse: f64 = xs[..n]
            .iter()
            .zip(&ys[..n])
            .map(|(x, y)| (y - intercept - slope * x).powi(2))
            .sum();
        let se = (sse / (nf - 2.0) / sxx).sqrt();
        let t = T975.get(n - 3).copied().unwrap_or(1.96);
        (slope - t * se, slope + t * se)
    });
    Some(Fit {
        slope,
        intercept,
        slope_ci,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_line() {
        let f = linear_fit(&[1.0, 2.0, 3.0], &[3.0, 5.0, 7.0]).unwrap();
        assert!((f.slope - 2.0).abs() < 1e-12 && (f.intercept - 1.0).abs() < 1e-12);
        let (lo, hi) = f.slope_ci.unwrap();
        assert!((lo - 2.0).abs() < 1e-9 && (hi - 2.0).abs() < 1e-9);
    }
}
