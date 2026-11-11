//! The resize view: a resize storm from 640×480 to 1200×800 in one second, and a scale switch
//! from 1 to 2 and back: configure to present, frames at a stale size, and memory at scale 2.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use leptos::prelude::*;

use crate::state::RunView;
use crate::views::kit::{DotsCard, ScopePicker, ViewHead, WithData};

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
        "fps",
        "Frames during the storm",
        "frames per second",
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
    PANELS
        .iter()
        .map(|(a, b, c, _, _)| MetricRef::new(a, b, c))
        .collect()
}

/// The resize view.
#[component]
pub fn ResizeView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead
                title="Resize"
                text="The rows app under a resize storm and a scale switch. Configure to present is how long a new size takes to reach the screen."
            />
            <RunView view={move |o: Arc<RunOverview>| view! {
                <WithData overview=o metrics=metrics() skeletons=4 view=move |data| {
                    let size = RwSignal::new(String::new());
                    let rate = RwSignal::new(String::new());
                    let d = data.clone();
                    view! {
                        <ScopePicker data=data.clone() scenarios=vec!["resize"] size=size rate=rate/>
                        <div class="gx-grid">
                            {PANELS.iter().map(|(a, b, c, title, words)| view! {
                                <DotsCard data=d.clone() title=*title words=*words metric=MetricRef::new(a, b, c) scenarios=vec![("resize", "resize")] size=size rate=rate/>
                            }).collect_view()}
                        </div>
                    }
                }/>
            }}/>
        </div>
    }
}
