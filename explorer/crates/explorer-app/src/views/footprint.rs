//! The footprint view: memory and VRAM against the row count with the memory one more row costs,
//! and what an idle app does in three seconds without input, the middle second suspended.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use explorer_core::stats::linear_fit;
use explorer_core::units::short;
use leptos::prelude::*;

use crate::chart::{Axis, ChartCard, XyModel, XyPlot, spread_note, use_card};
use crate::state::{RunView, Selection, use_explorer};
use crate::views::kit::{Build, Data, DotsCard, ScopePicker, ViewHead, WithData};

/// The memory panels: metric, title, axis words.
const MEMORY: [(&str, &str, &str); 3] = [
    ("pss_max", "Memory (PSS)", "MiB"),
    ("private_max", "Private memory", "MiB"),
    ("gpu_proc_mem_max", "VRAM", "MiB, per process"),
];

/// The idle panels: section, metric, title, axis words.
const IDLE: [(&str, &str, &str, &str); 6] = [
    (
        "resources",
        "cpu_ms_per_s",
        "Idle CPU time",
        "CPU ms per second",
    ),
    (
        "resources",
        "wakeups_per_s",
        "Idle wakeups",
        "context switches per second",
    ),
    (
        "idle_efficiency",
        "commits_per_s",
        "Commits while idle",
        "per second",
    ),
    (
        "idle_efficiency",
        "suspended_commits",
        "Commits while suspended",
        "count",
    ),
    (
        "idle_efficiency",
        "wasted_fraction",
        "Wasted commits",
        "fraction of commits that change nothing",
    ),
    ("resources", "threads_max", "Threads", "count"),
];

fn metrics() -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = MEMORY
        .iter()
        .map(|(m, _, _)| MetricRef::new("idle", "resources", m))
        .collect();
    out.extend(IDLE.iter().map(|(s, m, _, _)| MetricRef::new("idle", s, m)));
    out
}

/// The footprint view.
#[component]
pub fn FootprintView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead
                title="Memory & idle"
                text="The footprint after startup against the number of rows, and an idle app: three seconds with no input, the middle second suspended."
            />
            <RunView view={move |o: Arc<RunOverview>| view! {
                <WithData overview=o metrics=metrics() skeletons=3 view=move |data| {
                    let size = RwSignal::new(String::new());
                    let rate = RwSignal::new(String::new());
                    let d = data.clone();
                    view! {
                        <section class="gx-section">
                            <h2>"Memory against rows" <span>"labels: the memory one more row costs"</span></h2>
                            <div class="gx-grid">
                                {MEMORY.iter().map(|(m, title, words)| {
                                    let d = d.clone();
                                    view! {
                                        <ChartCard title=*title>
                                            <MemoryPlot data=d metric=MetricRef::new("idle", "resources", m) words=*words/>
                                        </ChartCard>
                                    }
                                }).collect_view()}
                            </div>
                        </section>
                        <section class="gx-section">
                            <h2>"Idle" <span>"rows scenario"</span></h2>
                            <ScopePicker data=d.clone() scenarios=vec!["rows"] size=size rate=rate/>
                            <div class="gx-grid">
                                {IDLE.iter().map(|(s, m, title, words)| view! {
                                    <DotsCard data=d.clone() title=*title words=*words metric=MetricRef::new("idle", s, m) scenarios=vec![("rows", "rows")] size=size rate=rate/>
                                }).collect_view()}
                            </div>
                        </section>
                    }
                }/>
            }}/>
        </div>
    }
}

#[component]
fn MemoryPlot(data: Data, metric: MetricRef, words: &'static str) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note.set(format!(
            "{} · lowest refresh rate",
            spread_note(false, ctx.uncertainty(), "Line")
        ));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let sel = Selection::new(data.overview(), &prefs, "rows");
        let hz = sel.rates().first().copied().unwrap_or(60.0);
        let ticks: Vec<f64> = sel.sizes().iter().map(|(r, _)| f64::from(*r)).collect();
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let mut model = XyModel::new(Axis::log_fixed("rows", ticks), Axis::value(words, "MiB"));
        model.height = 300.0;
        model.series = b.rows_series("rows", &metric, hz);
        for s in model.series.iter_mut().filter(|s| !s.ghost) {
            let xs: Vec<f64> = s.points.iter().map(|p| p.x).collect();
            let ys: Vec<f64> = s.points.iter().map(|p| p.y * 1024.0).collect();
            if let Some(fit) = linear_fit(&xs, &ys) {
                let ci = fit
                    .slope_ci
                    .map(|(lo, hi)| format!(" ({}–{})", short(lo), short(hi)))
                    .unwrap_or_default();
                s.label = format!("{} · {} KiB/row", s.label, short(fit.slope));
                for p in &mut s.points {
                    p.tip.push((
                        "per row".to_owned(),
                        format!("{} KiB{ci}", short(fit.slope)),
                    ));
                }
            }
        }
        model
    });
    view! { <XyPlot model=model/> }
}
