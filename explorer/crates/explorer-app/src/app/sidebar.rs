//! The sidebar: the framework and refresh rate filters, and the defaults of every
//! chart (scale, uncertainty, outlier focus, valid only, baseline).

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use crate::palette;
use crate::state::{Uncertainty, use_explorer};
use crate::widgets::{Chip, Segmented, SelectBox, Toggle};

/// The filters and chart defaults.
#[component]
pub fn Sidebar() -> impl IntoView {
    let explorer = use_explorer();
    view! {
        <Transition fallback=|| view! { <div class="gx-side__wait"></div> }>
            {move || Suspend::new(async move {
                explorer.overview.await.ok().map(|o| view! { <Filters overview=Arc::new(o)/> })
            })}
        </Transition>
        <ChartDefaults/>
        <div class="gx-side__help">
            <div class="gx-side__title">"Charts"</div>
            <p>"drag: zoom to a box · ctrl+wheel: zoom · ctrl+shift+wheel: zoom the value axis · shift+drag: pan · double-click: reset"</p>
            <p>"legend and framework list: click hides · alt+click shows alone · hover highlights in every chart · a point opens its cell"</p>
        </div>
    }
}

fn toggle_in(list: &[String], item: &str) -> Vec<String> {
    if list.iter().any(|x| x == item) {
        list.iter().filter(|x| *x != item).cloned().collect()
    } else {
        let mut out = list.to_vec();
        out.push(item.to_owned());
        out
    }
}

fn join(list: Vec<String>) -> Option<String> {
    (!list.is_empty()).then(|| list.join(","))
}

#[component]
fn Filters(overview: Arc<RunOverview>) -> impl IntoView {
    let explorer = use_explorer();
    let prefs = explorer.prefs;
    let mut frameworks = overview.frameworks();
    palette::sort(&mut frameworks);
    let all = frameworks.clone();
    let mut rates: Vec<f64> = Vec::new();
    for c in &overview.cells {
        if !rates.contains(&c.key.hz) {
            rates.push(c.key.hz);
        }
    }
    rates.sort_by(f64::total_cmp);
    let all_shown = {
        let all = all.clone();
        move || prefs.hidden.get().is_empty() || all.iter().all(|f| !prefs.hidden.get().contains(f))
    };
    let none_shown = {
        let all = all.clone();
        move || all.iter().all(|f| prefs.hidden.get().contains(f))
    };
    let hide_all = {
        let all = all.clone();
        move |_| explorer.query.set("hide", join(all.clone()))
    };
    view! {
        <section class="gx-side__group">
            <div class="gx-side__title">
                "Frameworks"
                <span class="gx-side__links">
                    <button type="button" class="gx-btn gx-btn--link" disabled=all_shown on:click=move |_| explorer.query.set("hide", None)>"all"</button>
                    <button type="button" class="gx-btn gx-btn--link" disabled=none_shown on:click=hide_all>"none"</button>
                </span>
            </div>
            <div class="gx-chips gx-chips--col">
                {frameworks
                    .into_iter()
                    .map(|fw| {
                        let name = fw.clone();
                        let solo_name = fw.clone();
                        let all = all.clone();
                        let on = Signal::derive({
                            let fw = fw.clone();
                            move || !prefs.hidden.get().contains(&fw)
                        });
                        view! {
                            <Chip
                                label=fw.clone()
                                color=palette::mark(&fw)
                                ink=palette::ink(&fw)
                                on=on
                                title="click: show or hide · alt+click: show alone"
                                on_toggle=Callback::new(move |_| {
                                    explorer.query.set("hide", join(toggle_in(&prefs.hidden.get_untracked(), &name)))
                                })
                                on_solo=Callback::new(move |_| {
                                    let rest: Vec<String> = all.iter().filter(|f| **f != solo_name).cloned().collect();
                                    explorer.query.set("hide", join(rest))
                                })
                            />
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        {(rates.len() > 1).then(|| view! {
            <section class="gx-side__group">
                <div class="gx-side__title">"Refresh rate" <span class="gx-side__hint">"the slower one holds pacing steps only"</span></div>
                <div class="gx-chips">
                    {rates.iter().map(|hz| {
                        let key = format!("{hz}");
                        let all: Vec<String> = rates.iter().map(|r| format!("{r}")).collect();
                        let on = Signal::derive({
                            let key = key.clone();
                            move || { let s = prefs.rates.get(); s.is_empty() || s.contains(&key) }
                        });
                        view! {
                            <Chip
                                label=format!("{hz} Hz")
                                on=on
                                on_toggle=Callback::new(move |_| {
                                    let mut shown = prefs.rates.get_untracked();
                                    if shown.is_empty() { shown = all.clone(); }
                                    let next = toggle_in(&shown, &key);
                                    let value = if next.len() == all.len() || next.is_empty() { None } else { join(next) };
                                    explorer.query.set("hz", value)
                                })
                            />
                        }
                    }).collect_view()}
                </div>
            </section>
        })}
        <section class="gx-side__group">
            <div class="gx-side__title">"Baseline" <span class="gx-side__hint">"values as ×"</span></div>
            <SelectBox
                options=Signal::derive(move || {
                    let mut o = vec![(String::new(), "none (absolute values)".to_owned())];
                    o.extend(all.iter().map(|f| (f.clone(), f.clone())));
                    o
                })
                value=Signal::derive(move || prefs.base.get().unwrap_or_default())
                on_change=Callback::new(move |v: String| explorer.query.set("base", Some(v)))
                title="Show every value relative to this framework"
            />
        </section>
    }
}

#[component]
fn ChartDefaults() -> impl IntoView {
    let explorer = use_explorer();
    let prefs = explorer.prefs;
    let opts = |pairs: &[(&str, &str)]| -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, l)| ((*k).to_owned(), (*l).to_owned()))
            .collect()
    };
    let scale_opts = opts(&[("", "auto"), ("lin", "linear"), ("log", "log")]);
    let unc_opts: Vec<(String, String)> = Uncertainty::ALL
        .iter()
        .map(|u| (u.key().to_owned(), u.label().to_owned()))
        .collect();
    view! {
        <section class="gx-side__group">
            <div class="gx-side__title">"Value scale"</div>
            <Segmented
                options=scale_opts
                value=Signal::derive(move || prefs.scale.get().key().to_owned())
                on_change=Callback::new(move |k: String| explorer.query.set("scale", Some(k)))
                title="The value axis of every chart; a chart may override it"
            />
        </section>
        <section class="gx-side__group">
            <div class="gx-side__title">"Uncertainty"</div>
            <SelectBox
                options=unc_opts
                value=Signal::derive(move || prefs.uncertainty.get().key().to_owned())
                on_change=Callback::new(move |k: String| explorer.query.set("unc", Some(k)))
                title="What whiskers and bands show. auto: middle half and p5–p95 of samples, or the iteration range"
            />
        </section>
        <section class="gx-side__group gx-side__toggles">
            <Toggle
                label="Focus: pin outliers to the edge"
                value=Signal::derive(move || prefs.focus.get())
                on_change=Callback::new(move |on: bool| explorer.query.set("focus", on.then(|| "1".to_owned())))
                title="Fit the value axis to the bulk of the series, so small differences stay visible"
            />
            <Toggle
                label="Valid reports only"
                value=Signal::derive(move || prefs.valid_only.get())
                on_change=Callback::new(move |on: bool| explorer.query.set("valid", on.then(|| "1".to_owned())))
                title="Hide values from reports that fail a validity check"
            />
        </section>
    }
}
