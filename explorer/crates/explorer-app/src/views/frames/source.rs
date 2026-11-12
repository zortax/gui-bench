//! The measurements with continuous frames: scroll glides, steady animation and resize storms,
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

/// Every source, in picker order.
pub const SOURCES: [Source; 7] = [
    Source {
        scenario: "rows",
        measurement: "scroll",
        label: "rows · scroll",
        words: "a 1 s scroll glide over a list with every row built",
    },
    Source {
        scenario: "virtual",
        measurement: "scroll",
        label: "virtual · scroll",
        words: "a 1 s scroll glide over a virtualized list",
    },
    Source {
        scenario: "dashboard",
        measurement: "scroll",
        label: "dashboard · scroll",
        words: "a 1 s scroll glide over the dashboard and its orders",
    },
    Source {
        scenario: "ticker",
        measurement: "pacing",
        label: "ticker",
        words: "3 s of table rows updating every 16 ms",
    },
    Source {
        scenario: "spinner",
        measurement: "pacing",
        label: "spinner",
        words: "3 s of a turning square above static rows",
    },
    Source {
        scenario: "resize",
        measurement: "resize",
        label: "resize · storm",
        words: "a storm of window resizes of the plain list",
    },
    Source {
        scenario: "dashboard-resize",
        measurement: "resize",
        label: "dashboard · resize",
        words: "a storm of window resizes of the dashboard",
    },
];

impl Source {
    /// Returns the URL key, such as `rows.scroll`.
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
    out.push(source.metric("stale_refreshes"));
    out
}
