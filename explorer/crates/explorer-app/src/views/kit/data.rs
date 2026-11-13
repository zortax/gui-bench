//! Loading the metric columns of a view: the columns of the run and, with a compare run, the
//! same columns of that run. Loading shows skeleton cards; a failure shows an error card with a
//! retry.

use std::collections::HashMap;
use std::sync::Arc;

use explorer_core::api::{CellId, CellInfo, CellKey, MetricColumn, MetricRef, RunOverview, Stat};
use leptos::prelude::*;

use crate::api::{fetch_columns, metric_columns, metric_list, remember};
use crate::state::use_explorer;
use crate::widgets::{ChartSkeleton, ErrorCard};

/// The columns of one run.
#[derive(Debug, Clone)]
pub struct RunCols {
    pub overview: Arc<RunOverview>,
    pub cols: Arc<HashMap<MetricRef, MetricColumn>>,
}

impl RunCols {
    /// Returns a column.
    pub fn col(&self, m: &MetricRef) -> Option<&MetricColumn> {
        self.cols.get(m)
    }

    /// Returns the statistic of a metric in a cell.
    pub fn stat(&self, m: &MetricRef, cell: CellId) -> Option<&Stat> {
        self.col(m)?.of(cell)
    }

    /// Returns the cell of the same framework at the same matrix point.
    pub fn cell_of(&self, key: &CellKey) -> Option<&CellInfo> {
        self.overview
            .cells
            .iter()
            .find(|c| c.key.framework == key.framework && c.key.same_point(key))
    }
}

/// The data of a view: the run, and the compare run when one is set.
#[derive(Debug, Clone)]
pub struct Data {
    pub run: RunCols,
    pub compare: Option<RunCols>,
}

impl Data {
    pub fn overview(&self) -> &RunOverview {
        &self.run.overview
    }

    /// Returns the statistic of a metric in a cell of the run.
    pub fn stat(&self, m: &MetricRef, cell: CellId) -> Option<&Stat> {
        self.run.stat(m, cell)
    }

    /// Returns the statistic of a metric in the compare run's cell with the same key.
    pub fn ghost(&self, m: &MetricRef, key: &CellKey) -> Option<&Stat> {
        let c = self.compare.as_ref()?;
        c.stat(m, c.cell_of(key)?.id)
    }

    /// Returns true when the values of a metric are per-repeat samples.
    pub fn samples(&self, m: &MetricRef) -> bool {
        self.run.col(m).is_some_and(|c| c.samples)
    }

    /// Returns the unit of a metric.
    pub fn unit(&self, m: &MetricRef) -> String {
        self.run.col(m).map(|c| c.unit.clone()).unwrap_or_default()
    }
}

fn index(cols: Vec<MetricColumn>) -> Arc<HashMap<MetricRef, MetricColumn>> {
    Arc::new(cols.into_iter().map(|c| (c.metric.clone(), c)).collect())
}

/// Loads the columns of metrics and renders a view from them once they are there. The columns
/// load on the server with the page and travel inside it; a later change of run, metrics or
/// compare run fetches only the columns that change.
#[component]
pub fn WithData<F, V>(
    overview: Arc<RunOverview>,
    #[prop(into)] metrics: Signal<Vec<MetricRef>>,
    view: F,
    /// The skeleton cards while loading.
    #[prop(optional)]
    skeletons: Option<usize>,
    /// Load the cells of these frameworks only; empty or absent loads every framework.
    #[prop(optional, into)]
    frameworks: Option<Signal<Vec<String>>>,
) -> impl IntoView
where
    F: Fn(Data) -> V + Send + Sync + Clone + 'static,
    V: IntoView + 'static,
{
    let explorer = use_explorer();
    let run = overview.summary.id.clone();
    let version = overview.summary.modified_unix.unwrap_or(0);
    let columns = Resource::new(
        move || {
            (
                run.clone(),
                version,
                metric_list(&metrics.get()),
                explorer.prefs.compare.get(),
                frameworks.map(|f| f.get()).unwrap_or_default(),
            )
        },
        |(run, version, list, compare, only)| async move {
            let refs: Vec<MetricRef> = list.split(',').filter_map(MetricRef::parse).collect();
            let mine = fetch_columns(run, version, refs, only.clone()).await?;
            let theirs = match compare {
                Some(id) => Some(metric_columns(id, list, only.join(",")).await?),
                None => None,
            };
            Ok::<_, ServerFnError>((mine, theirs))
        },
    );
    let skeletons = skeletons.unwrap_or(2);
    let fallback = move || {
        (0..skeletons)
            .map(|_| view! { <ChartSkeleton/> })
            .collect_view()
    };
    view! {
        <Transition fallback=fallback>
            {move || {
                let view = view.clone();
                let overview = overview.clone();
                Suspend::new(async move {
                    let loaded = columns.await;
                    let compare = explorer.compare.await.and_then(Result::ok);
                    match loaded {
                        Ok((mine, theirs)) => {
                            if frameworks.is_none_or(|f| f.get_untracked().is_empty()) {
                                let summary = &overview.summary;
                                remember(&summary.id, summary.modified_unix.unwrap_or(0), &mine);
                            }
                            let run = RunCols { overview, cols: index(mine) };
                            let compare = compare.zip(theirs).map(|(o, cols)| RunCols { overview: Arc::new(o), cols: index(cols) });
                            view(Data { run, compare }).into_any()
                        }
                        Err(e) => view! {
                            <ErrorCard
                                title="The metrics do not load"
                                error=e.to_string()
                                retry=Callback::new(move |_| columns.refetch())
                            />
                        }
                        .into_any(),
                    }
                })
            }}
        </Transition>
    }
}
