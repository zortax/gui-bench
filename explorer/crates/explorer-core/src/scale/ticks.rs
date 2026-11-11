//! Nice ticks: steps of 1, 2 and 5 times a power of ten on linear axes, and decades with their
//! 2 and 5 multiples on log axes.

/// A tick of an axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    pub value: f64,
    /// The tick carries a label and a grid line; a minor tick carries a grid line only.
    pub major: bool,
}

/// Returns the nice step for a span divided into about `count` parts.
fn nice_step(span: f64, count: usize) -> f64 {
    let raw = span / count.max(1) as f64;
    let power = 10f64.powf(raw.log10().floor());
    let unit = raw / power;
    let nice = if unit < 1.5 {
        1.0
    } else if unit < 3.5 {
        2.0
    } else if unit < 7.5 {
        5.0
    } else {
        10.0
    };
    nice * power
}

/// Returns about `count` ticks between `lo` and `hi` on a linear axis.
pub fn linear_ticks(lo: f64, hi: f64, count: usize) -> Vec<Tick> {
    if !(lo.is_finite() && hi.is_finite()) || hi <= lo {
        return Vec::new();
    }
    let step = nice_step(hi - lo, count);
    let first = (lo / step).ceil() as i64;
    let last = (hi / step).floor() as i64;
    (first..=last)
        .map(|i| {
            let v = i as f64 * step;
            Tick {
                value: if v.abs() < step * 1e-9 { 0.0 } else { v },
                major: true,
            }
        })
        .collect()
}

/// Returns ticks between `lo` and `hi` on a log axis: decades, their 2 and 5 multiples when the
/// span is short, and linear ticks when the span holds less than one decade.
pub fn log_ticks(lo: f64, hi: f64, count: usize) -> Vec<Tick> {
    if !(lo > 0.0 && hi > lo && hi.is_finite()) {
        return Vec::new();
    }
    let decades = (hi / lo).log10();
    if decades < 1.0 {
        return linear_ticks(lo, hi, count.min(5));
    }
    let first = lo.log10().floor() as i32;
    let last = hi.log10().ceil() as i32;
    let every = ((decades / count.max(1) as f64).ceil() as i32).max(1);
    let mults: &[f64] = if decades <= 2.5 {
        &[1.0, 2.0, 5.0]
    } else if decades <= 4.5 {
        &[1.0, 3.0]
    } else {
        &[1.0]
    };
    let mut out = Vec::new();
    for d in first..=last {
        let base = 10f64.powi(d);
        for &m in mults {
            let v = base * m;
            if v >= lo * (1.0 - 1e-9) && v <= hi * (1.0 + 1e-9) {
                let major = (m == 1.0 && (d - first) % every == 0) || every == 1;
                out.push(Tick { value: v, major });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_steps() {
        let t: Vec<f64> = linear_ticks(0.0, 176.0, 4)
            .iter()
            .map(|t| t.value)
            .collect();
        assert_eq!(t, vec![0.0, 50.0, 100.0, 150.0]);
    }

    #[test]
    fn log_decades() {
        let t: Vec<f64> = log_ticks(10.0, 10000.0, 6)
            .iter()
            .map(|t| t.value)
            .collect();
        assert_eq!(t, vec![10.0, 30.0, 100.0, 300.0, 1000.0, 3000.0, 10000.0]);
        let short = log_ticks(12.0, 18.0, 5);
        assert!(!short.is_empty());
    }
}
