//! The server functions. Each reads the run store from context on a blocking thread. They use
//! GET requests, so browsers and proxies may cache their answers.

use explorer_core::api as wire;
use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

#[cfg(feature = "ssr")]
mod backend {
    use std::sync::Arc;

    use explorer_data::{RunData, Store};
    use leptos::prelude::*;

    /// Runs a closure on the store on a blocking thread.
    pub async fn with_store<T: Send + 'static>(
        f: impl FnOnce(&Store) -> Result<T, String> + Send + 'static,
    ) -> Result<T, ServerFnError> {
        let store = use_context::<Arc<Store>>()
            .ok_or_else(|| ServerFnError::new("the server has no run store"))?;
        tokio::task::spawn_blocking(move || f(&store))
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .map_err(ServerFnError::new)
    }

    /// Runs a closure on a loaded run on a blocking thread.
    pub async fn with_run<T: Send + 'static>(
        run: String,
        f: impl FnOnce(&RunData) -> Result<T, explorer_data::DataError> + Send + 'static,
    ) -> Result<T, ServerFnError> {
        with_store(move |store| {
            let data = store.run(&run).map_err(|e| e.to_string())?;
            f(&data).map_err(|e| e.to_string())
        })
        .await
    }
}

/// Returns every run of the results directory, newest first.
#[server(input = GetUrl)]
pub async fn list_runs() -> Result<Vec<wire::RunSummary>, ServerFnError> {
    backend::with_store(|store| Ok(store.runs())).await
}

/// Returns the provenance, cells and metric catalog of a run.
#[server(input = GetUrl)]
pub async fn run_overview(run: String) -> Result<wire::RunOverview, ServerFnError> {
    backend::with_run(run, |data| Ok(data.overview.clone())).await
}

/// Returns every metric of a run with its unit, direction and scenarios.
#[server(input = GetUrl)]
pub async fn metric_catalog(run: String) -> Result<Vec<wire::MetricInfo>, ServerFnError> {
    backend::with_run(run, |data| Ok(data.catalog.clone())).await
}

/// Returns the columns of metrics, given as comma-separated `measurement.section.metric`, with
/// the cells of the comma-separated frameworks only, or of every framework without any.
#[server(input = GetUrl)]
pub async fn metric_columns(
    run: String,
    metrics: String,
    #[server(default)] frameworks: String,
) -> Result<Vec<wire::MetricColumn>, ServerFnError> {
    let refs: Vec<_> = metrics
        .split(',')
        .filter_map(explorer_core::api::MetricRef::parse)
        .collect();
    let frameworks: Vec<String> = frameworks
        .split(',')
        .filter(|f| !f.is_empty())
        .map(str::to_owned)
        .collect();
    backend::with_run(run, move |data| Ok(data.columns_of(&refs, &frameworks))).await
}

/// Returns the detail of a cell.
#[server(input = GetUrl)]
pub async fn cell_detail(run: String, cell: u32) -> Result<wire::CellDetail, ServerFnError> {
    backend::with_run(run, move |data| data.cell_detail(cell)).await
}

/// Returns the latency series of a measurement of a cell.
#[server(input = GetUrl)]
pub async fn latency_detail(
    run: String,
    cell: u32,
    measurement: String,
) -> Result<wire::LatencyDetail, ServerFnError> {
    backend::with_run(run, move |data| data.latency_detail(cell, &measurement)).await
}

/// Returns the frame series of a measurement of a cell.
#[server(input = GetUrl)]
pub async fn frames_detail(
    run: String,
    cell: u32,
    measurement: String,
) -> Result<wire::FramesDetail, ServerFnError> {
    backend::with_run(run, move |data| data.frames_detail(cell, &measurement)).await
}

/// Returns the typed keys of a measurement of a cell.
#[server(input = GetUrl)]
pub async fn typing_detail(
    run: String,
    cell: u32,
    measurement: String,
) -> Result<wire::TypingDetail, ServerFnError> {
    backend::with_run(run, move |data| data.typing_detail(cell, &measurement)).await
}

/// Returns the configures of a resize measurement of a cell.
#[server(input = GetUrl)]
pub async fn resize_detail(
    run: String,
    cell: u32,
    measurement: String,
) -> Result<wire::ResizeDetail, ServerFnError> {
    backend::with_run(run, move |data| data.resize_detail(cell, &measurement)).await
}

/// Returns the app phases of a measurement of a cell, or `None` when the app reports none.
#[server(input = GetUrl)]
pub async fn phases_detail(
    run: String,
    cell: u32,
    measurement: String,
) -> Result<Option<wire::PhasesDetail>, ServerFnError> {
    backend::with_run(run, move |data| {
        match data.phases_detail(cell, &measurement) {
            Ok(d) => Ok(Some(d)),
            Err(explorer_data::DataError::NoReport(_)) => Ok(None),
            Err(e) => Err(e),
        }
    })
    .await
}

/// Returns the frame profiles of a measurement in every cell of a matrix point. `updates` is the
/// ticker updates per tick, zero for other scenarios.
#[server(input = GetUrl)]
pub async fn frame_profiles(
    run: String,
    scenario: String,
    measurement: String,
    hz: f64,
    rows: u32,
    #[server(default)] updates: u32,
) -> Result<Vec<wire::FrameProfile>, ServerFnError> {
    let point = explorer_data::Point {
        scenario,
        hz,
        rows,
        updates: (updates > 0).then_some(updates),
    };
    backend::with_run(run, move |data| {
        Ok(data.frame_profiles(&point, &measurement))
    })
    .await
}
