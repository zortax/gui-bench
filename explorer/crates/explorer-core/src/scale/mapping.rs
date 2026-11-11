//! A scale maps a data domain onto a pixel range, linearly or in log space, and zooms and pans
//! that domain.

use serde::{Deserialize, Serialize};

use super::ticks::{Tick, linear_ticks, log_ticks};

/// How a scale maps values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleKind {
    Linear,
    Log,
}

/// A map from a data domain to a pixel range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    pub kind: ScaleKind,
    /// The data domain; the lower bound may exceed the upper one.
    pub domain: (f64, f64),
    /// The pixel range; it may run backwards, as a y axis does.
    pub range: (f64, f64),
}

/// The smallest positive value a log scale accepts.
const LOG_FLOOR: f64 = 1e-9;

impl Scale {
    /// Returns a scale. A log scale clamps its domain to positive values.
    pub fn new(kind: ScaleKind, domain: (f64, f64), range: (f64, f64)) -> Self {
        let domain = match kind {
            ScaleKind::Linear if domain.0 == domain.1 => (domain.0 - 1.0, domain.1 + 1.0),
            ScaleKind::Linear => domain,
            ScaleKind::Log => {
                let lo = domain.0.max(LOG_FLOOR);
                let hi = domain.1.max(LOG_FLOOR);
                if lo == hi {
                    (lo / 2.0, hi * 2.0)
                } else {
                    (lo, hi)
                }
            }
        };
        Self {
            kind,
            domain,
            range,
        }
    }

    fn forward(&self, v: f64) -> f64 {
        match self.kind {
            ScaleKind::Linear => v,
            ScaleKind::Log => v.max(LOG_FLOOR).log10(),
        }
    }

    fn backward(&self, t: f64) -> f64 {
        match self.kind {
            ScaleKind::Linear => t,
            ScaleKind::Log => 10f64.powf(t),
        }
    }

    /// Returns the pixel position of a value.
    pub fn map(&self, v: f64) -> f64 {
        let (a, b) = (self.forward(self.domain.0), self.forward(self.domain.1));
        let t = (self.forward(v) - a) / (b - a);
        self.range.0 + t * (self.range.1 - self.range.0)
    }

    /// Returns the value at a pixel position.
    pub fn invert(&self, px: f64) -> f64 {
        let (a, b) = (self.forward(self.domain.0), self.forward(self.domain.1));
        let t = (px - self.range.0) / (self.range.1 - self.range.0);
        self.backward(a + t * (b - a))
    }

    /// Returns true when a value lies inside the domain.
    pub fn contains(&self, v: f64) -> bool {
        let (lo, hi) = self.bounds();
        v >= lo && v <= hi
    }

    /// Returns the domain as (low, high).
    pub fn bounds(&self) -> (f64, f64) {
        if self.domain.0 <= self.domain.1 {
            self.domain
        } else {
            (self.domain.1, self.domain.0)
        }
    }

    /// Returns about `count` ticks inside the domain.
    pub fn ticks(&self, count: usize) -> Vec<Tick> {
        let (lo, hi) = self.bounds();
        match self.kind {
            ScaleKind::Linear => linear_ticks(lo, hi, count),
            ScaleKind::Log => log_ticks(lo, hi, count),
        }
    }

    /// Returns the domain zoomed by `factor` around the value at pixel `at`. A factor below 1
    /// zooms in.
    pub fn zoomed(&self, at: f64, factor: f64) -> (f64, f64) {
        let center = self.forward(self.invert(at));
        let (a, b) = (self.forward(self.domain.0), self.forward(self.domain.1));
        (
            self.backward(center + (a - center) * factor),
            self.backward(center + (b - center) * factor),
        )
    }

    /// Returns the domain moved so that content follows a drag of `dpx` pixels.
    pub fn panned(&self, dpx: f64) -> (f64, f64) {
        let (a, b) = (self.forward(self.domain.0), self.forward(self.domain.1));
        let shift = -dpx / (self.range.1 - self.range.0) * (b - a);
        (self.backward(a + shift), self.backward(b + shift))
    }

    /// Returns the domain between two pixel positions, in domain order.
    pub fn between(&self, p0: f64, p1: f64) -> (f64, f64) {
        let (a, b) = (self.invert(p0), self.invert(p1));
        if (a <= b) == (self.domain.0 <= self.domain.1) {
            (a, b)
        } else {
            (b, a)
        }
    }
}

/// Returns a padded domain around the data: a linear domain starts at zero when the data is
/// positive and near it, and a log domain widens by a small factor.
pub fn padded(kind: ScaleKind, lo: f64, hi: f64, from_zero: bool) -> (f64, f64) {
    match kind {
        ScaleKind::Linear => {
            let lo = if from_zero && lo >= 0.0 { 0.0 } else { lo };
            let span = (hi - lo).abs().max(f64::EPSILON);
            (
                if from_zero && lo == 0.0 {
                    0.0
                } else {
                    lo - span * 0.05
                },
                hi + span * 0.06,
            )
        }
        ScaleKind::Log => {
            let lo = lo.max(LOG_FLOOR);
            let hi = hi.max(lo * 1.0001);
            let ratio = (hi / lo).powf(0.06).max(1.05);
            (lo / ratio, hi * ratio)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_and_inverts() {
        let s = Scale::new(ScaleKind::Log, (1.0, 1000.0), (0.0, 300.0));
        assert!((s.map(10.0) - 100.0).abs() < 1e-9);
        assert!((s.invert(200.0) - 100.0).abs() < 1e-9);
        let y = Scale::new(ScaleKind::Linear, (0.0, 10.0), (100.0, 0.0));
        assert!((y.map(5.0) - 50.0).abs() < 1e-9);
    }

    #[test]
    fn zoom_keeps_center() {
        let s = Scale::new(ScaleKind::Linear, (0.0, 100.0), (0.0, 100.0));
        let (a, b) = s.zoomed(50.0, 0.5);
        assert!((a - 25.0).abs() < 1e-9 && (b - 75.0).abs() < 1e-9);
        let (a, b) = s.panned(10.0);
        assert!((a + 10.0).abs() < 1e-9 && (b - 90.0).abs() < 1e-9);
    }
}
