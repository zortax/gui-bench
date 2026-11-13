//! The cost of a larger app: the least-squares slope of a response over the size, in
//! milliseconds per 1,000 elements.

use crate::stats::linear_fit;

/// The slopes in ms per 1,000 elements that take the lightest and the darkest shade.
pub const SLOPE_LOW: f64 = 0.01;
pub const SLOPE_HIGH: f64 = 30.0;

/// Returns the slope of `(elements, ms)` points in ms per 1,000 elements, or `None` with fewer
/// than two sizes.
pub fn slope_per_thousand(points: &[(f64, f64)]) -> Option<f64> {
    let xs: Vec<f64> = points.iter().map(|(x, _)| x / 1000.0).collect();
    let ys: Vec<f64> = points.iter().map(|(_, y)| *y).collect();
    linear_fit(&xs, &ys).map(|f| f.slope)
}

/// Returns the text and the shade (0 to 1) of a slope.
pub fn slope_shade(slope: f64) -> (String, f64) {
    if slope < SLOPE_LOW {
        return (format!("< {SLOPE_LOW}"), 0.05);
    }
    let t = (slope.ln() - SLOPE_LOW.ln()) / (SLOPE_HIGH.ln() - SLOPE_LOW.ln());
    let text = if slope < 10.0 {
        format!("{slope:.2}")
    } else {
        format!("{slope:.0}")
    };
    (text, 0.05 + 0.95 * t.min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_thousand() {
        let s = slope_per_thousand(&[(1000.0, 1.0), (3000.0, 5.0)]).unwrap();
        assert!((s - 2.0).abs() < 1e-12);
        assert_eq!(slope_per_thousand(&[(1000.0, 1.0)]), None);
        assert_eq!(slope_shade(2.0).0, "2.00");
        assert_eq!(slope_shade(0.001).0, "< 0.01");
        assert_eq!(slope_shade(12.4).0, "12");
    }
}
