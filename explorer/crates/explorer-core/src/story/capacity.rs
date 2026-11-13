//! The capacity of a framework for one step: the size up to which every measured value passes
//! a budget. Between the last size that passes and the first that fails, a response budget
//! interpolates on log axes.

/// How far a step stays within its budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Capacity {
    /// The smallest measured size fails already.
    Below(f64),
    /// The size at which the value crosses the budget.
    At(f64),
    /// Every measured size passes; the largest one is the bound.
    AtLeast(f64),
}

impl Capacity {
    /// Returns the size.
    pub fn size(self) -> f64 {
        match self {
            Self::Below(s) | Self::At(s) | Self::AtLeast(s) => s,
        }
    }

    /// Returns the text of the capacity, such as `< 558`, `3.5k` or `≥ 11k`.
    pub fn text(self) -> String {
        let size = crate::units::compact(self.size());
        match self {
            Self::Below(_) => format!("< {size}"),
            Self::At(_) => size,
            Self::AtLeast(_) => format!("≥ {size}"),
        }
    }
}

/// Returns the capacity over `(size, value)` points in ascending size, or `None` without
/// points. A value passes when it is at most `limit`. `interpolate` places the crossing between
/// two sizes; without it the capacity is the last size that passes. `open` says that the
/// framework ran every size, so a pass at the largest size is a lower bound.
pub fn capacity(
    points: &[(f64, f64)],
    limit: f64,
    interpolate: bool,
    open: bool,
) -> Option<Capacity> {
    let first = points.first()?;
    let held = points.iter().take_while(|(_, v)| *v <= limit).count();
    if held == 0 {
        return Some(Capacity::Below(first.0));
    }
    if held == points.len() {
        let last = points[held - 1].0;
        return Some(if open {
            Capacity::AtLeast(last)
        } else {
            Capacity::At(last)
        });
    }
    let ((x0, y0), (x1, y1)) = (points[held - 1], points[held]);
    if !interpolate || y0 <= 0.0 || y1 <= y0 {
        return Some(Capacity::At(x0));
    }
    let t = (limit.ln() - y0.ln()) / (y1.ln() - y0.ln());
    Some(Capacity::At((x0.ln() + t * (x1.ln() - x0.ln())).exp()))
}

/// Returns the place of a size between the smallest and the largest measured size on a log
/// scale, from 0.12 to 1. A capacity below the smallest size takes 0.
pub fn shade(capacity: Capacity, smallest: f64, largest: f64) -> f64 {
    if matches!(capacity, Capacity::Below(_)) {
        return 0.0;
    }
    let (lo, hi) = (smallest.ln(), largest.ln());
    let t = if hi > lo {
        (capacity.size().ln() - lo) / (hi - lo)
    } else {
        1.0
    };
    0.12 + 0.88 * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds() {
        assert_eq!(capacity(&[], 1.0, true, true), None);
        let p = [(558.0, 2.0), (11228.0, 4.0)];
        assert_eq!(capacity(&p, 1.0, true, true), Some(Capacity::Below(558.0)));
        assert_eq!(
            capacity(&p, 5.0, true, true),
            Some(Capacity::AtLeast(11228.0))
        );
        assert_eq!(capacity(&p, 5.0, true, false), Some(Capacity::At(11228.0)));
        assert_eq!(capacity(&p, 3.0, false, true), Some(Capacity::At(558.0)));
    }

    #[test]
    fn crossing_on_log_axes() {
        let p = [(100.0, 1.0), (10000.0, 100.0)];
        let c = capacity(&p, 10.0, true, true).unwrap();
        assert!((c.size() - 1000.0).abs() < 1e-6);
        assert_eq!(c.text(), "1k");
        assert!((shade(c, 100.0, 10000.0) - 0.56).abs() < 1e-9);
    }
}
