//! A client cache of metric columns per run and run version (the last write of its index), so
//! switching views asks the server only for columns it has not sent yet, and a run that grows
//! gets fresh columns.

use std::cell::RefCell;
use std::collections::HashMap;

use explorer_core::api::{MetricColumn, MetricRef};
use leptos::prelude::ServerFnError;

/// Columns by run id, run version and metric.
type Cache = HashMap<((String, u64), MetricRef), MetricColumn>;

thread_local! {
    static COLUMNS: RefCell<Cache> = RefCell::new(HashMap::new());
}

/// Joins metrics into the list form of [`super::metric_columns`].
pub fn metric_list(metrics: &[MetricRef]) -> String {
    metrics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// Returns the columns of metrics of a run, in the order asked, from the cache where it can.
/// Columns of some frameworks only (`frameworks` not empty) bypass the cache.
pub async fn fetch_columns(
    run: String,
    version: u64,
    metrics: Vec<MetricRef>,
    frameworks: Vec<String>,
) -> Result<Vec<MetricColumn>, ServerFnError> {
    if cfg!(feature = "ssr") || !frameworks.is_empty() {
        // The server answers from its run cache; a per-thread copy would only grow.
        return super::metric_columns(run, metric_list(&metrics), frameworks.join(",")).await;
    }
    let key = (run.clone(), version);
    let missing: Vec<MetricRef> = COLUMNS.with_borrow(|cache| {
        metrics
            .iter()
            .filter(|m| !cache.contains_key(&(key.clone(), (*m).clone())))
            .cloned()
            .collect()
    });
    if !missing.is_empty() {
        let fetched =
            super::metric_columns(run.clone(), metric_list(&missing), String::new()).await?;
        COLUMNS.with_borrow_mut(|cache| {
            for column in fetched {
                cache.insert((key.clone(), column.metric.clone()), column);
            }
        });
    }
    Ok(COLUMNS.with_borrow(|cache| {
        metrics
            .iter()
            .filter_map(|m| cache.get(&(key.clone(), m.clone())).cloned())
            .collect()
    }))
}

/// Keeps columns that arrived another way, such as inside the server-rendered page, so a later
/// view of the same run finds them. The server keeps nothing.
pub fn remember(run: &str, version: u64, columns: &[MetricColumn]) {
    if cfg!(feature = "ssr") {
        return;
    }
    COLUMNS.with_borrow_mut(|cache| {
        for column in columns {
            cache
                .entry(((run.to_owned(), version), column.metric.clone()))
                .or_insert_with(|| column.clone());
        }
    });
}
