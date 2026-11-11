//! The run controls of the top bar: the run picker with the catalog, the compare run, and the
//! provenance of the run.

use std::sync::Arc;

use explorer_core::api::{RunOverview, RunStatus, RunSummary};
use explorer_core::units::duration;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::state::use_explorer;
use crate::widgets::{Badge, Icon, IconKind, LocalTime, Popover, StatusDot};

/// The run picker, the compare chip and the provenance button.
#[component]
pub fn RunBar() -> impl IntoView {
    let explorer = use_explorer();
    view! {
        <div class="gx-runbar">
            <Popover
                class="gx-runbar__run"
                title="Pick a run"
                button=ViewFn::from(move || view! {
                    <span class="gx-runbar__name">{move || explorer.run.get()}</span>
                    <Icon kind=IconKind::ChevronDown/>
                })
            >
                <RunList/>
            </Popover>
            <Transition fallback=|| ()>
                {move || Suspend::new(async move {
                    explorer.overview.await.ok().map(|o| view! { <RunFacts overview=Arc::new(o)/> })
                })}
            </Transition>
            {move || explorer.prefs.compare.get().map(|cmp| view! {
                <span class="gx-runbar__cmp" title="Values of this run show as ghost marks">
                    "vs " <b>{cmp}</b>
                    <button
                        type="button"
                        class="gx-iconbtn gx-iconbtn--tiny"
                        title="Stop comparing runs"
                        on:click=move |_| explorer.query.set("cmp", None)
                    >
                        <Icon kind=IconKind::Close/>
                    </button>
                </span>
            })}
        </div>
    }
}

/// The date, size, health and provenance of the run.
#[component]
fn RunFacts(overview: Arc<RunOverview>) -> impl IntoView {
    let s = &overview.summary;
    let p = &overview.provenance;
    let span = p
        .started_unix
        .zip(p.modified_unix)
        .map(|(a, b)| duration(b.saturating_sub(a)));
    let invalid = p.validity.reports - p.validity.valid;
    let overview2 = overview.clone();
    view! {
        <span class="gx-runbar__facts">
            {p.started_unix.map(|t| view! { <LocalTime unix=t/> })}
            {span.map(|d| view! { <span class="gx-muted">{format!("· {d}")}</span> })}
            <span class="gx-muted">{format!("· {} cells", s.cells)}</span>
            {(s.failed_cells > 0).then(|| view! {
                <Badge text=format!("{} failed", s.failed_cells) tone="err" title="Cells with a failed measurement"/>
            })}
            {(invalid > 0).then(|| view! {
                <Badge
                    text=format!("{invalid}/{} invalid", p.validity.reports)
                    tone="warn"
                    title="Reports that fail a validity check, such as compositor wake lateness"
                />
            })}
            {s.live.then(|| view! { <Badge text="live" tone="info" title="The runner wrote this run a moment ago"/> })}
        </span>
        <Popover
            right=false
            class="gx-iconbtn"
            title="Provenance of the run"
            button=ViewFn::from(|| view! { <Icon kind=IconKind::Info/> })
        >
            <crate::views::Provenance overview=overview2.clone()/>
        </Popover>
    }
}

/// The catalog of runs in the picker.
#[component]
fn RunList() -> impl IntoView {
    let explorer = use_explorer();
    view! {
        <div class="gx-runlist">
            <Suspense fallback=|| view! { <div class="gx-runlist__wait">"Reading the results…"</div> }>
                {move || Suspend::new(async move {
                    match explorer.runs.await {
                        Ok(runs) => runs
                            .into_iter()
                            .map(|r| view! { <RunRow run=r/> })
                            .collect_view()
                            .into_any(),
                        Err(e) => view! { <div class="gx-runlist__wait">{e.to_string()}</div> }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn RunRow(run: RunSummary) -> impl IntoView {
    let explorer = use_explorer();
    let location = use_location();
    let id = run.id.clone();
    let current = {
        let id = id.clone();
        let aliases = run.aliases.clone();
        move || {
            let r = explorer.run.get();
            r == id || aliases.contains(&r)
        }
    };
    let href = {
        let id = id.clone();
        move || {
            let path = location.pathname.get();
            let view = path
                .trim_start_matches('/')
                .split_once('/')
                .map(|(_, v)| v.to_owned())
                .unwrap_or_default();
            let search = location.search.get();
            let search = if search.is_empty() {
                String::new()
            } else {
                format!("?{}", search.trim_start_matches('?'))
            };
            format!("/{id}/{view}{search}")
        }
    };
    let unsupported = match &run.status {
        RunStatus::Unsupported(reason) => Some(reason.clone()),
        RunStatus::Ok => None,
    };
    let readable = unsupported.is_none();
    let cmp_id = id.clone();
    view! {
        <div class="gx-runrow" data-current=move || current().then_some("") data-off=(!readable).then_some("")>
            {if readable {
                view! { <a class="gx-runrow__name" href=href>{id.clone()}</a> }.into_any()
            } else {
                view! { <span class="gx-runrow__name" title=unsupported.clone()>{id.clone()}</span> }.into_any()
            }}
            {run.aliases.iter().map(|a| view! { <Badge text=a.clone() tone="info"/> }).collect_view()}
            {run.live.then(|| view! { <Badge text="live" tone="info"/> })}
            <span class="gx-runrow__meta">
                {run.started_unix.map(|t| view! { <LocalTime unix=t/> })}
                <span>{format!("{} cells", run.cells)}</span>
                {(run.failed_cells > 0).then(|| view! {
                    <span class="gx-runrow__fail"><StatusDot status=explorer_core::api::CheckStatus::Fail/>{run.failed_cells}</span>
                })}
                <span class="gx-muted">{format!("{} scenarios · {} frameworks", run.scenarios.len(), run.frameworks.len())}</span>
            </span>
            {match unsupported {
                Some(reason) => view! { <span class="gx-runrow__why">{reason}</span> }.into_any(),
                None => view! {
                    <button
                        type="button"
                        class="gx-btn gx-btn--small"
                        title="Show this run as ghost marks in every chart"
                        on:click=move |_| explorer.query.set("cmp", Some(cmp_id.clone()))
                    >
                        "compare"
                    </button>
                }.into_any(),
            }}
        </div>
    }
}
