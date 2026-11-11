//! App frame phases: the span marks that an app attaches to its commits, such as layout, paint
//! and GPU submit, grouped by name with their durations.

use std::collections::BTreeMap;

use fakeland::{Event, Trace};
use fakeland_trace::event::AppMarkKind;
use serde::Serialize;

/// The durations of one phase.
#[derive(Debug, Default, Serialize)]
pub struct Phase {
    /// The number of spans.
    pub count: usize,
    /// The median duration in milliseconds.
    pub p50_ms: f64,
    /// The 95th percentile in milliseconds.
    pub p95_ms: f64,
    /// The mean in milliseconds.
    pub mean_ms: f64,
    /// The longest span in milliseconds.
    pub max_ms: f64,
    /// Every duration in milliseconds, in trace order.
    pub durations_ms: Vec<f64>,
}

/// Collects the span marks of every trace by phase name.
pub fn collect<'a>(traces: impl IntoIterator<Item = &'a Trace>) -> BTreeMap<String, Phase> {
    let mut durations: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for trace in traces {
        for record in &trace.records {
            if let Event::AppMark {
                mark: AppMarkKind::Span { name, begin, end },
                ..
            } = &record.event
            {
                durations
                    .entry(name.clone())
                    .or_default()
                    .push(end.since(*begin).as_millis_f64());
            }
        }
    }
    durations
        .into_iter()
        .map(|(name, values)| (name, summarize(values)))
        .collect()
}

fn summarize(values: Vec<f64>) -> Phase {
    let mut sorted = values.clone();
    sorted.sort_by(f64::total_cmp);
    let at = |q: f64| {
        sorted
            .get(((sorted.len() as f64 - 1.0) * q).round() as usize)
            .copied()
            .unwrap_or(0.0)
    };
    Phase {
        count: values.len(),
        p50_ms: at(0.5),
        p95_ms: at(0.95),
        mean_ms: values.iter().sum::<f64>() / values.len().max(1) as f64,
        max_ms: sorted.last().copied().unwrap_or(0.0),
        durations_ms: values,
    }
}
