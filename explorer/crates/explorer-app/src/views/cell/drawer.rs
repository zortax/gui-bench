//! The drawer frame: it opens for the `cell` query parameter, closes on Escape or a click
//! outside, and loads the cell detail.

use explorer_core::api::{CellDetail, CheckStatus};
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
    let names: Vec<(String, String)> = detail
        .measurements
        .iter()
        .map(|m| {
            let mark = if m.valid { "" } else { " ⚠" };
            (m.name.clone(), format!("{}{mark}", m.name))
        })
        .collect();
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
    let shot = format!("/files/{run}/{id}/screenshot.png");
    view! {
        <header class="gx-drawer__head">
            <h2 style:color=palette::ink(&k.framework)>{k.label()}</h2>
            <p>{format!("{} · {} · {} Hz", k.scenario, k.size(), k.hz)}</p>
            <div class="gx-drawer__badges">
                <Badge text=format!("{} done", cell.done.len()) tone="ok"/>
                {(!cell.errors.is_empty()).then(|| view! { <Badge text=format!("{} failed", cell.errors.len()) tone="err"/> })}
                {(!cell.skipped.is_empty()).then(|| view! { <Badge text=format!("{} skipped", cell.skipped.len()) tone="muted"/> })}
            </div>
            <code class="gx-drawer__args">{format!("{}-bench {}", k.framework, detail.args.join(" "))}</code>
        </header>
        {cell.errors.iter().map(|(m, e)| view! {
            <div class="gx-drawer__error"><StatusDot status=CheckStatus::Fail/>" "<b>{m.clone()}</b>": "{e.clone()}</div>
        }).collect_view()}
        {cell.skipped.iter().map(|(m, why)| view! {
            <div class="gx-drawer__skip"><b>{m.clone()}</b>" skipped: "{why.clone()}</div>
        }).collect_view()}
        <div class="gx-drawer__top">
            {detail.has_screenshot.then(|| view! {
                <a class="gx-drawer__shot" href=shot.clone() target="_blank" title="Open the screenshot">
                    <img src=shot.clone() alt="The app as the layout check saw it" loading="lazy"/>
                </a>
            })}
            {detail.startup.clone().map(|s| view! {
                <dl class="gx-kv gx-drawer__startup">
                    {s.milestones().into_iter().map(|(name, v)| view! {
                        <dt>{name}</dt><dd>{v.map_or("–".to_owned(), |v| format!("{} ms", short(v)))}</dd>
                    }).collect_view()}
                    {s.exited_early.then(|| view! { <dt>"exit"</dt><dd>"the app exited early"</dd> })}
                </dl>
            })}
        </div>
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
