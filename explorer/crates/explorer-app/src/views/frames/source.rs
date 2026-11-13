//! The measurements with continuous frames: scrolls, live updates and resize storms,
//! each in a scenario, and the frame metrics the views read of them.

use explorer_core::api::{MetricRef, RunOverview};

/// A measurement with continuous frames in one scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source {
    pub scenario: &'static str,
    pub measurement: &'static str,
    /// The name in the source picker.
    pub label: &'static str,
    /// What happens during the measurement.
    pub words: &'static str,
}

const fn source(
    scenario: &'static str,
    measurement: &'static str,
    label: &'static str,
    words: &'static str,
) -> Source {
    Source {
        scenario,
        measurement,
        label,
        words,
    }
}

/// Every source, in picker order.
pub const SOURCES: [Source; 10] = [
    source(
        "app",
        "scroll",
        "app · glide",
        "a 1 s touchpad glide over the orders of the app",
    ),
    source(
        "app",
        "scroll-fling",
        "app · fling",
        "a fling over the orders of the app",
    ),
    source(
        "app",
        "scroll-wheel",
        "app · wheel",
        "1 s of mouse wheel notches over the orders of the app",
    ),
    source(
        "app",
        "live",
        "app · live",
        "the live values ticking at 10 Hz beside the turning spinner",
    ),
    source(
        "app",
        "resize",
        "app · resize",
        "a storm of window resizes of the overview",
    ),
    source(
        "app",
        "resize-messages",
        "app · resize messages",
        "a storm of window resizes over the wrapping text of the messages",
    ),
    source(
        "app",
        "scale",
        "app · scale",
        "a switch of the output scale 1, 2, 1",
    ),
    source(
        "list",
        "scroll",
        "list · glide",
        "a 1 s touchpad glide over the virtual list",
    ),
    source(
        "list",
        "scroll-fling",
        "list · fling",
        "a fling over the virtual list",
    ),
    source(
        "list",
        "scroll-wheel",
        "list · wheel",
        "1 s of mouse wheel notches over the virtual list",
    ),
];

impl Source {
    /// Returns the URL key, such as `app.scroll-fling`.
    pub fn key(&self) -> String {
        format!("{}.{}", self.scenario, self.measurement)
    }

    /// Returns the source of a key, or the first one.
    pub fn parse(key: &str) -> Source {
        SOURCES
            .iter()
            .copied()
            .find(|s| s.key() == key)
            .unwrap_or(SOURCES[0])
    }

    /// Returns the source of a measurement in a scenario, or the first one.
    pub fn of(scenario: &str, measurement: &str) -> Source {
        Self::parse(&format!("{scenario}.{measurement}"))
    }

    /// Returns a frame metric of the source, such as `interval_p99`.
    pub fn metric(&self, name: &str) -> MetricRef {
        MetricRef::new(self.measurement, "frame_pacing", name)
    }

    /// Returns true when some cell of the run took the measurement.
    pub fn present(&self, overview: &RunOverview) -> bool {
        overview.cells.iter().any(|c| {
            c.key.scenario == self.scenario && c.done.iter().any(|d| d == self.measurement)
        })
    }
}

/// The interval percentiles in ladder order, with their names.
pub const LADDER: [(&str, &str); 6] = [
    ("interval_p50", "p50"),
    ("interval_p90", "p90"),
    ("interval_p95", "p95"),
    ("interval_p99", "p99"),
    ("interval_p999", "p99.9"),
    ("interval_max", "max"),
];

/// Returns the frame metrics of a source that the views read.
pub fn metrics(source: &Source) -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = LADDER.iter().map(|(m, _)| source.metric(m)).collect();
    out.push(source.metric("missed_share"));
    out
}
