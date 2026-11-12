//! The resize view: a resize storm from 640×480 to 1200×800 in one second, and a scale switch
//! from 1 to 2 and back, of the plain list or the dashboard: frame times during the storm,
//! configure to present, frames at a stale size, and memory at scale 2.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use leptos::prelude::*;

use crate::state::{RunView, use_param};
use crate::views::frames::{ProfileCard, SOURCES};
use crate::views::kit::{ScopeBar, ScopeDotsCard, ViewHead, WithData, use_scope};
use crate::widgets::Segmented;

/// The panels: measurement, section, metric, title, axis words.
const PANELS: [(&str, &str, &str, &str, &str); 9] = [
    (
        "resize",
        "resize",
        "configure_to_present_p50",
        "Resize: configure to present",
        "median, milliseconds",
    ),
    (
        "resize",
        "resize",
        "configure_to_present_max",
        "Resize: slowest configure",
        "milliseconds",
    ),
    (
        "resize",
        "resize",
        "mismatched_frames",
        "Frames at a stale size",
        "count per storm",
    ),
    (
        "resize",
        "resize",
        "missed_vblanks",
        "Missed refreshes during the storm",
        "count",
    ),
    (
        "resize",
        "frame_pacing",
        "interval_max",
        "Longest frame gap during the storm",
        "ms between two presents, worst",
    ),
    (
        "resize",
        "resources",
        "cpu_ms_per_s",
        "App CPU during the storm",
        "CPU ms per second",
    ),
    (
        "scale",
        "resize",
        "configure_to_present_p50",
        "Scale switch: configure to present",
        "median, milliseconds",
    ),
    (
        "scale",
        "resources",
        "pss_max",
        "Memory at scale 2",
        "PSS, MiB",
    ),
    (
        "scale",
        "resources",
        "gpu_proc_mem_max",
        "VRAM at scale 2",
        "MiB, per process",
    ),
];

fn metrics() -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = PANELS
        .iter()
        .map(|(a, b, c, _, _)| MetricRef::new(a, b, c))
        .collect();
    out.extend(crate::views::frames::metrics(&SOURCES[5]));
    out
}

/// The resize view.
#[component]
pub fn ResizeView() -> impl IntoView {
    let (scenario, set_scenario) = use_param("sc", "resize");
    let options = vec![
        ("resize".to_owned(), "plain list".to_owned()),
        ("dashboard-resize".to_owned(), "dashboard".to_owned()),
    ];
    view! {
        <div class="gx-view">
            <ViewHead
                title="Resize"
                text="An app under a resize storm and a scale switch: the plain list, or the dashboard with its cards and chart. Configure to present is how long a new size takes to reach the screen."
            >
                <Segmented options=options value=Signal::derive(move || scenario.get()) on_change=set_scenario/>
            </ViewHead>
            <RunView view={move |o: Arc<RunOverview>| {
                let scope = use_scope(o.clone(), Signal::derive(move || scenario.get()));
                let source = Signal::derive(move || {
                    let sc = scenario.get();
                    SOURCES.iter().copied().find(|s| s.scenario == sc).unwrap_or(SOURCES[0])
                });
                view! {
                    <div class="gx-view__controls"><ScopeBar scope=scope/></div>
                    <WithData overview=o metrics=metrics() skeletons=4 view=move |data| {
                        let d = data.clone();
                        view! {
                            <div class="gx-grid gx-grid--wide">
                                {move || view! {
                                    <ProfileCard data=d.clone() source=source.get() scope=scope per_refresh=Signal::stored(false) title="Frame times during the storm"/>
                                }}
                            </div>
                            <div class="gx-grid">
                                {PANELS.iter().map(|(a, b, c, title, words)| view! {
                                    <ScopeDotsCard data=data.clone() title=*title words=*words metric=MetricRef::new(a, b, c) scenario=scenario scope=scope/>
                                }).collect_view()}
                            </div>
                        }
                    }/>
                }
            }}/>
        </div>
    }
}
