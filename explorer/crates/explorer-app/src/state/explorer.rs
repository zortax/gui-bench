//! The explorer context: the run of the route, the run catalog, the overview of the run and of
//! the compare run, the preferences and the query writer. `RunView` renders a view once the
//! overview is there and keeps the previous view up while another run loads.

use std::sync::Arc;

use explorer_core::api::{RunOverview, RunSummary};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use super::{Prefs, QueryWriter};
use crate::api::{list_runs, run_overview};
use crate::widgets::{ErrorCard, ViewSkeleton};

/// The shared state of the explorer pages.
#[derive(Clone, Copy)]
pub struct Explorer {
    /// The run id or alias of the route.
    pub run: Memo<String>,
    pub runs: Resource<Result<Vec<RunSummary>, ServerFnError>>,
    pub overview: Resource<Result<RunOverview, ServerFnError>>,
    /// The overview of the compare run, `None` without one.
    pub compare: Resource<Option<Result<RunOverview, ServerFnError>>>,
    pub prefs: Prefs,
    pub query: QueryWriter,
    /// Another run loads while the previous view stays up.
    pub pending: RwSignal<bool>,
    /// The framework under the pointer in any chart; every chart highlights it.
    pub highlight: RwSignal<Option<String>>,
    /// The source line of exported charts: the run, its revisions and the host.
    pub footnote: RwSignal<String>,
}

/// Provides the explorer context for the run of the route.
pub fn provide_explorer() -> Explorer {
    let params = use_params_map();
    let run = Memo::new(move |_| {
        params
            .with(|p| p.get("run"))
            .unwrap_or_else(|| "latest".to_owned())
    });
    let prefs = Prefs::from_query();
    let runs = Resource::new(|| (), |_| list_runs());
    let overview = Resource::new(move || run.get(), run_overview);
    let compare = Resource::new(
        move || prefs.compare.get(),
        |cmp| async move {
            match cmp {
                Some(id) => Some(run_overview(id).await),
                None => None,
            }
        },
    );
    let explorer = Explorer {
        run,
        runs,
        overview,
        compare,
        prefs,
        query: QueryWriter::new(),
        pending: RwSignal::new(false),
        highlight: RwSignal::new(None),
        footnote: RwSignal::new(String::new()),
    };
    provide_context(explorer);
    explorer
}

/// Returns the explorer context.
pub fn use_explorer() -> Explorer {
    expect_context::<Explorer>()
}

/// Renders a view from the run overview, with a skeleton while the first run loads and the
/// previous view while another run loads.
#[component]
pub fn RunView<F, V>(view: F) -> impl IntoView
where
    F: Fn(Arc<RunOverview>) -> V + Send + Sync + Clone + 'static,
    V: IntoView + 'static,
{
    let explorer = use_explorer();
    view! {
        <Transition fallback=ViewSkeleton set_pending=explorer.pending>
            {move || {
                let view = view.clone();
                Suspend::new(async move {
                    match explorer.overview.await {
                        Ok(overview) => {
                            explorer.footnote.set(footnote(&overview));
                            view(Arc::new(overview)).into_any()
                        }
                        Err(e) => view! { <ErrorCard title="The run does not load" error=e.to_string()/> }.into_any(),
                    }
                })
            }}
        </Transition>
    }
}

/// Returns the source line of a run: its id, revisions and host.
fn footnote(overview: &RunOverview) -> String {
    let p = &overview.provenance;
    let mut parts = vec![format!("run {}", overview.summary.id)];
    for r in &p.revisions {
        if let Some(c) = &r.commit {
            parts.push(format!(
                "{} {c}{}",
                r.name,
                if r.dirty { "+dirty" } else { "" }
            ));
        }
    }
    if let Some(env) = &p.environment {
        parts.push(format!("{}, {}", env.cpu, env.gpu));
    }
    format!("gui-bench under fakeland · {}", parts.join(" · "))
}
