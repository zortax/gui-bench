//! The drawer frame: it opens for the `cell` query parameter, closes on Escape or a click
//! outside, and loads the cell detail: the outcome, the hand-built parts, the screenshots, the
//! launches and one panel per measurement.

use explorer_core::api::{CellDetail, CheckStatus, Part};
use explorer_core::story::part_of;
use explorer_core::units::short;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use super::measurement::MeasurementPanel;
use crate::api::cell_detail;
use crate::palette;
use crate::state::use_explorer;
use crate::widgets::{Badge, ChartSkeleton, ErrorCard, Icon, IconKind, Segmented, StatusDot};

/// The drawer of the open cell, if any.
#[component]
pub fn CellDrawer() -> impl IntoView {
    let explorer = use_explorer();
    move || {
        explorer
            .prefs
            .cell
            .get()
            .map(|id| view! { <Drawer id=id/> })
    }
}

#[component]
fn Drawer(id: u32) -> impl IntoView {
    let explorer = use_explorer();
    let run = explorer.run;
    let detail = Resource::new(move || (run.get(), id), |(run, id)| cell_detail(run, id));
    let close = move || explorer.query.set("cell", None);
    let handle = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            close();
        }
    });
    on_cleanup(move || handle.remove());
    view! {
        <div class="gx-drawer__scrim" on:click=move |_| close()></div>
        <aside class="gx-drawer" role="dialog" aria-label="Cell detail">
            <button type="button" class="gx-iconbtn gx-drawer__close" title="Close (Escape)" on:click=move |_| close()>
                <Icon kind=IconKind::Close/>
            </button>
            <Suspense fallback=|| view! { <ChartSkeleton/><ChartSkeleton/> }>
                {move || Suspend::new(async move {
                    match detail.await {
                        Ok(d) => view! { <Detail detail=d/> }.into_any(),
                        Err(e) => view! { <ErrorCard title="The cell does not load" error=e.to_string()/> }.into_any(),
                    }
                })}
            </Suspense>
        </aside>
    }
}

async fn fetch_text(url: String) -> Result<String, String> {
    let response = wasm_bindgen_futures::JsFuture::from(window().fetch_with_str(&url))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let response: web_sys::Response = response.dyn_into().map_err(|e| format!("{e:?}"))?;
    let text = wasm_bindgen_futures::JsFuture::from(response.text().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(text.as_string().unwrap_or_default())
}

#[component]
fn Detail(detail: CellDetail) -> impl IntoView {
    let explorer = use_explorer();
    let run = explorer.run.get_untracked();
    let cell = detail.cell.clone();
    let k = &cell.key;
    let id = cell.id;
    let flags = detail.flags.clone();
    let scenario = k.scenario.clone();
    let hand_built = move |measurement: &str| {
        part_of(&scenario, measurement)
            .and_then(|part| flags.iter().find(|(p, _)| p == part))
            .cloned()
    };
    let names: Vec<(String, String)> = detail
        .measurements
        .iter()
        .map(|m| {
            let invalid = if m.valid { "" } else { " ⚠" };
            let hollow = if hand_built(&m.name).is_some() {
                " ○"
            } else {
                ""
            };
            (m.name.clone(), format!("{}{hollow}{invalid}", m.name))
        })
        .collect();
    let parts: Vec<(String, String, Vec<String>)> = detail
        .flags
        .iter()
        .map(|(part, note)| {
            let steps = detail
                .measurements
                .iter()
                .filter(|m| part_of(&k.scenario, &m.name) == Some(part.as_str()))
                .map(|m| m.name.clone())
                .collect();
            (part.clone(), note.clone(), steps)
        })
        .filter(|(_, _, steps): &(String, String, Vec<String>)| !steps.is_empty())
        .collect();
    let invalid = detail.measurements.iter().filter(|m| !m.valid).count();
    let first = names.first().map(|(n, _)| n.clone()).unwrap_or_default();
    let picked = RwSignal::new(first);
    let measurements = StoredValue::new(detail.measurements.clone());
    let log_open = RwSignal::new(false);
    let log_url = format!("/files/{run}/{id}/app.log");
    let log = LocalResource::new(move || {
        let open = log_open.get();
        let url = log_url.clone();
        async move {
            if open {
                fetch_text(url).await
            } else {
                Ok(String::new())
            }
        }
    });
    let shots: Vec<(String, String)> = detail
        .screenshots
        .iter()
        .map(|stem| {
            let what = match stem.as_str() {
                "screenshot" => "after startup",
                "screenshot-menu" => "the menu open",
                "screenshot-live" => "the live updates on",
                "screenshot-messages" => "the messages page",
                other => other,
            };
            (format!("/files/{run}/{id}/{stem}.png"), what.to_owned())
        })
        .collect();
    let launches = detail.launches.clone();
    view! {
        <header class="gx-drawer__head">
            <h2 style:color=palette::ink(&k.framework)>{k.framework.clone()}</h2>
            <p>{format!("{} · {} · {} Hz", k.scenario, k.size_text(), k.hz)}</p>
            <div class="gx-drawer__badges">
                {palette::is_edition(&k.framework).then(|| view! { <Badge text=format!("edition of {}", palette::base(&k.framework)) tone="info"/> })}
                {(cell.part == Part::Pacing).then(|| view! { <Badge text="pacing steps only" tone="info" title="The cell missed refreshes at the rate of every cell, so its pacing steps run again at the slower rate"/> })}
                {cell.stopped.is_some().then(|| view! { <Badge text="not run" tone="warn"/> })}
                {cell.over_budget.then(|| view! { <Badge text="over budget" tone="warn" title="A response took over a second or never came, so the later steps did not run"/> })}
                {cell.missed_refreshes.then(|| view! { <Badge text="missed refreshes" tone="warn" title="A pacing step missed more than 1 % of the refreshes"/> })}
                {(invalid > 0).then(|| view! { <Badge text=format!("{invalid} invalid") tone="warn" title="Reports that the host disturbed: a validity check fails"/> })}
                <Badge text=format!("{} done", cell.done.len()) tone="ok"/>
                {(!cell.errors.is_empty()).then(|| view! { <Badge text=format!("{} failed", cell.errors.len()) tone="err"/> })}
                {(!cell.skipped.is_empty()).then(|| view! { <Badge text=format!("{} skipped", cell.skipped.len()) tone="muted"/> })}
            </div>
            <code class="gx-drawer__args">{format!("{}-bench {}", k.framework, detail.args.join(" "))}</code>
        </header>
        {cell.stopped.clone().map(|why| view! {
            <div class="gx-drawer__skip"><b>"not run: "</b>{why}</div>
        })}
        {parts.into_iter().map(|(part, note, steps)| view! {
            <div class="gx-drawer__skip"><Badge text=part tone="warn"/>" built by hand: "{note}<span class="gx-muted">{format!(" · {}", steps.join(", "))}</span></div>
        }).collect_view()}
        {cell.errors.iter().map(|(m, e)| view! {
            <div class="gx-drawer__error"><StatusDot status=CheckStatus::Fail/>" "<b>{m.clone()}</b>": "{e.clone()}</div>
        }).collect_view()}
        {cell.skipped.iter().map(|(m, why)| view! {
            <div class="gx-drawer__skip"><b>{m.clone()}</b>" skipped: "{why.clone()}</div>
        }).collect_view()}
        <div class="gx-drawer__shots">
            {shots.into_iter().map(|(src, what)| {
                let title = format!("Open the screenshot: {what}");
                let alt = format!("The app, {what}");
                let href = src.clone();
                view! {
                    <a class="gx-drawer__shot" href=href target="_blank" title=title>
                        <img src=src alt=alt loading="lazy"/>
                        <span class="gx-muted">{what}</span>
                    </a>
                }
            }).collect_view()}
        </div>
        {(!launches.is_empty()).then(|| view! {
            <section class="gx-drawer__startup">
                <h4>"Startup" <span class="gx-muted">" · ms from spawn, one row per launch"</span></h4>
                <table class="gx-table">
                    <thead>
                        <tr>
                            <th>"#"</th>
                            {launches[0].milestones().into_iter().map(|(name, _)| view! { <th>{name}</th> }).collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {launches.iter().enumerate().map(|(i, l)| view! {
                            <tr>
                                <td>{i + 1}</td>
                                {l.milestones().into_iter().map(|(_, v)| view! { <td>{v.map_or("–".to_owned(), short)}</td> }).collect_view()}
                                {l.exited_early.then(|| view! { <td>"the app exited early"</td> })}
                            </tr>
                        }).collect_view()}
                    </tbody>
                </table>
            </section>
        })}
        <div class="gx-drawer__tabs">
            <Segmented options=names value=Signal::derive(move || picked.get()) on_change=Callback::new(move |v| picked.set(v))/>
        </div>
        {move || {
            let name = picked.get();
            measurements
                .with_value(|ms| ms.iter().find(|m| m.name == name).cloned())
                .map(|m| view! { <MeasurementPanel cell=id measurement=m/> })
        }}
        <details class="gx-drawer__log" on:toggle=move |_| log_open.set(true)>
            <summary>"app.log"</summary>
            <Suspense fallback=|| view! { <p class="gx-muted">"Loading…"</p> }>
                {move || Suspend::new(async move {
                    match log.await {
                        Ok(text) if text.is_empty() => view! { <p class="gx-muted">"empty"</p> }.into_any(),
                        Ok(text) => view! { <pre>{text}</pre> }.into_any(),
                        Err(e) => view! { <p class="gx-muted">{e}</p> }.into_any(),
                    }
                })}
            </Suspense>
        </details>
    }
}
