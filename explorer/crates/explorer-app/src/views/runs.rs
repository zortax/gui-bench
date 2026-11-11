//! The runs view: every run of the results directory with its provenance, and the differences
//! between the run and the compare run: revisions, host, output, effort, frameworks and
//! scenarios.

use std::sync::Arc;

use explorer_core::api::{RunOverview, RunStatus};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::state::use_explorer;
use crate::views::Provenance;
use crate::views::kit::ViewHead;
use crate::widgets::{Badge, ChartSkeleton, LocalTime};

/// The runs view.
#[component]
pub fn RunsView() -> impl IntoView {
    let explorer = use_explorer();
    view! {
        <div class="gx-view">
            <ViewHead title="Runs" text="Every run in the results directory. Open one, or compare it with the current run: its values show as ghost marks in every chart."/>
            <section class="gx-card">
                <Suspense fallback=|| view! { <ChartSkeleton/> }>
                    {move || Suspend::new(async move {
                        let runs = explorer.runs.await.unwrap_or_default();
                        view! { <RunTable runs=runs/> }
                    })}
                </Suspense>
            </section>
            <Transition fallback=|| view! { <ChartSkeleton/> }>
                {move || Suspend::new(async move {
                    let a = explorer.overview.await.ok()?;
                    let b = explorer.compare.await.and_then(Result::ok);
                    Some(match b {
                        Some(b) => view! { <Diff a=Arc::new(a) b=Arc::new(b)/> }.into_any(),
                        None => view! {
                            <section class="gx-card">
                                <h3 class="gx-chart__title">"Provenance of the current run"</h3>
                                <Provenance overview=Arc::new(a)/>
                            </section>
                        }
                        .into_any(),
                    })
                })}
            </Transition>
        </div>
    }
}

#[component]
fn RunTable(runs: Vec<explorer_core::api::RunSummary>) -> impl IntoView {
    let explorer = use_explorer();
    let location = use_location();
    view! {
        <table class="gx-table">
            <thead>
                <tr><th>"run"</th><th>"started"</th><th>"cells"</th><th>"failed"</th><th>"scenarios"</th><th>"frameworks"</th><th></th></tr>
            </thead>
            <tbody>
                {runs.into_iter().map(|r| {
                    let id = r.id.clone();
                    let cmp = r.id.clone();
                    let readable = r.readable();
                    let current = {
                        let id = id.clone();
                        let aliases = r.aliases.clone();
                        move || { let x = explorer.run.get(); x == id || aliases.contains(&x) }
                    };
                    let href = {
                        let id = id.clone();
                        move || format!("/{id}/runs{}", location.search.get().split_once('?').map_or(String::new(), |(_, q)| format!("?{q}")))
                    };
                    view! {
                        <tr data-current=move || current().then_some("")>
                            <td>
                                {if readable { view! { <a href=href><b>{id.clone()}</b></a> }.into_any() } else { view! { <span class="gx-muted">{id.clone()}</span> }.into_any() }}
                                {r.aliases.iter().map(|a| view! { " " <Badge text=a.clone() tone="info"/> }).collect_view()}
                                {r.live.then(|| view! { " " <Badge text="live" tone="info"/> })}
                                {match &r.status { RunStatus::Unsupported(why) => Some(view! { " " <Badge text="unsupported" tone="muted" title=why.clone()/> }), RunStatus::Ok => None }}
                            </td>
                            <td>{r.started_unix.map(|t| view! { <LocalTime unix=t/> })}</td>
                            <td>{r.cells}</td>
                            <td>{(r.failed_cells > 0).then(|| view! { <Badge text=r.failed_cells.to_string() tone="err"/> })}</td>
                            <td class="gx-muted">{r.scenarios.join(", ")}</td>
                            <td class="gx-muted">{r.frameworks.join(", ")}</td>
                            <td>
                                {readable.then(|| view! {
                                    <button type="button" class="gx-btn gx-btn--small" on:click=move |_| explorer.query.set("cmp", Some(cmp.clone()))>"compare"</button>
                                })}
                            </td>
                        </tr>
                    }
                }).collect_view()}
            </tbody>
        </table>
    }
}

/// The differences of two runs, row by row.
#[component]
fn Diff(a: Arc<RunOverview>, b: Arc<RunOverview>) -> impl IntoView {
    let pa = &a.provenance;
    let pb = &b.provenance;
    let rev = |p: &explorer_core::api::Provenance, name: &str| {
        p.revisions
            .iter()
            .find(|r| r.name == name)
            .map(|r| {
                format!(
                    "{}{}",
                    r.commit.clone().unwrap_or_default(),
                    if r.dirty { "+dirty" } else { "" }
                )
            })
            .unwrap_or_default()
    };
    let mut names: Vec<String> = pa
        .revisions
        .iter()
        .chain(&pb.revisions)
        .map(|r| r.name.clone())
        .collect();
    names.sort();
    names.dedup();
    let ea = pa.environment.clone().unwrap_or_default();
    let eb = pb.environment.clone().unwrap_or_default();
    let effort = |p: &explorer_core::api::Provenance| {
        format!(
            "{:?} it · {:?} hover · {:?} op · {:?} churn",
            p.effort.iterations, p.effort.hover_repeats, p.effort.op_repeats, p.effort.churn_cycles
        )
        .replace("Some(", "")
        .replace(')', "")
        .replace("None", "–")
    };
    let set = |x: &[String], y: &[String]| -> (String, String) {
        let only_x: Vec<&String> = x.iter().filter(|v| !y.contains(v)).collect();
        let only_y: Vec<&String> = y.iter().filter(|v| !x.contains(v)).collect();
        (
            if only_x.is_empty() {
                "same".to_owned()
            } else {
                format!(
                    "+ {}",
                    only_x
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            if only_y.is_empty() {
                "same".to_owned()
            } else {
                format!(
                    "+ {}",
                    only_y
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
        )
    };
    let mut rows: Vec<(String, String, String)> = Vec::new();
    for n in names {
        rows.push((format!("revision {n}"), rev(pa, &n), rev(pb, &n)));
    }
    rows.push(("cpu".into(), ea.cpu.clone(), eb.cpu.clone()));
    rows.push(("gpu".into(), ea.gpu.clone(), eb.gpu.clone()));
    rows.push(("driver".into(), ea.driver.clone(), eb.driver.clone()));
    rows.push(("kernel".into(), ea.kernel.clone(), eb.kernel.clone()));
    rows.push(("governor".into(), ea.governor.clone(), eb.governor.clone()));
    rows.push((
        "fakeland".into(),
        ea.fakeland_version.clone(),
        eb.fakeland_version.clone(),
    ));
    rows.push(("effort".into(), effort(pa), effort(pb)));
    rows.push((
        "cells".into(),
        a.summary.cells.to_string(),
        b.summary.cells.to_string(),
    ));
    rows.push((
        "valid reports".into(),
        format!("{} of {}", pa.validity.valid, pa.validity.reports),
        format!("{} of {}", pb.validity.valid, pb.validity.reports),
    ));
    let (fa, fb) = set(&pa.frameworks, &pb.frameworks);
    rows.push(("frameworks only here".into(), fa, fb));
    let (sa, sb) = set(&pa.scenarios, &pb.scenarios);
    rows.push(("scenarios only here".into(), sa, sb));
    view! {
        <section class="gx-card">
            <h3 class="gx-chart__title">"Run against run"</h3>
            <p class="gx-chart__note">"Rows that differ are marked; a difference in revisions, host or effort can explain a difference in values."</p>
            <table class="gx-table gx-diff">
                <thead><tr><th></th><th>{a.summary.id.clone()}</th><th>{b.summary.id.clone()}</th></tr></thead>
                <tbody>
                    {rows.into_iter().map(|(k, x, y)| {
                        let differs = x != y && !(x == "same" && y == "same");
                        view! { <tr data-differs=differs.then_some("")><td class="gx-muted">{k}</td><td>{x}</td><td>{y}</td></tr> }
                    }).collect_view()}
                </tbody>
            </table>
        </section>
    }
}
