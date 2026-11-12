//! Frame gaps by the refreshes they miss: the classes of a late frame, and the share of frames
//! and of time in each class over the iterations of a measurement.

use crate::api::GapLane;

/// A class of frames by the refreshes they miss.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GapClass {
    /// The fewest missed refreshes of the class.
    pub from: u32,
    /// The name, such as `2–3 late`.
    pub name: &'static str,
}

/// The classes, from on time to a freeze.
pub const GAP_CLASSES: [GapClass; 5] = [
    GapClass {
        from: 0,
        name: "on time",
    },
    GapClass {
        from: 1,
        name: "1 refresh late",
    },
    GapClass {
        from: 2,
        name: "2–3 late",
    },
    GapClass {
        from: 4,
        name: "4–9 late",
    },
    GapClass {
        from: 10,
        name: "10+ late",
    },
];

/// Returns the class index of a frame that missed `missed` refreshes.
pub fn gap_class(missed: u32) -> usize {
    GAP_CLASSES
        .iter()
        .rposition(|c| missed >= c.from)
        .unwrap_or(0)
}

/// The frames and the time of every class.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GapBreakdown {
    pub frames: [u32; GAP_CLASSES.len()],
    pub time_ms: [f64; GAP_CLASSES.len()],
}

impl GapBreakdown {
    /// Returns the frames of every class over all frames.
    pub fn frame_shares(&self) -> [f64; GAP_CLASSES.len()] {
        let total: u32 = self.frames.iter().sum();
        self.frames.map(|n| f64::from(n) / f64::from(total.max(1)))
    }

    /// Returns the time of every class over all time.
    pub fn time_shares(&self) -> [f64; GAP_CLASSES.len()] {
        let total: f64 = self.time_ms.iter().sum();
        self.time_ms
            .map(|t| if total > 0.0 { t / total } else { 0.0 })
    }
}

/// Returns the breakdown over lanes, or `None` when a lane lacks some late frames.
pub fn gap_breakdown(lanes: &[GapLane]) -> Option<GapBreakdown> {
    if lanes.is_empty() || lanes.iter().any(|l| !l.complete) {
        return None;
    }
    let mut out = GapBreakdown::default();
    for lane in lanes {
        let mut late_frames = 0;
        let mut late_ms = 0.0;
        for g in lane.gaps.iter().filter(|g| g.missed > 0) {
            let class = gap_class(g.missed);
            out.frames[class] += 1;
            out.time_ms[class] += g.interval_ms;
            late_frames += 1;
            late_ms += g.interval_ms;
        }
        out.frames[0] += lane.frames.saturating_sub(late_frames);
        let on_time = lane.total_ms - late_ms;
        if on_time > 1e-6 * lane.total_ms {
            out.time_ms[0] += on_time;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GapFrame;

    fn gap(interval_ms: f64, missed: u32) -> GapFrame {
        GapFrame {
            t_ms: 0.0,
            interval_ms,
            missed,
            stalled: false,
            double_step: false,
        }
    }

    #[test]
    fn classes_split_by_missed_refreshes() {
        assert_eq!(gap_class(0), 0);
        assert_eq!(gap_class(1), 1);
        assert_eq!(gap_class(3), 2);
        assert_eq!(gap_class(4), 3);
        assert_eq!(gap_class(40), 4);
    }

    #[test]
    fn time_counts_the_late_intervals() {
        let lane = GapLane {
            index: 0,
            window_ms: 1500.0,
            presents: Some((40.0, 1040.0)),
            frames: 10,
            total_ms: 200.0,
            complete: true,
            threshold_ms: 25.0,
            gaps: vec![gap(33.3, 1), gap(66.7, 3)],
        };
        let b = gap_breakdown(std::slice::from_ref(&lane)).unwrap();
        assert_eq!(b.frames, [8, 1, 1, 0, 0]);
        assert!((b.time_ms[0] - 100.0).abs() < 1e-9);
        assert!((b.time_shares()[2] - 66.7 / 200.0).abs() < 1e-9);
        let partial = GapLane {
            complete: false,
            ..lane
        };
        assert!(gap_breakdown(&[partial]).is_none());
    }
}
