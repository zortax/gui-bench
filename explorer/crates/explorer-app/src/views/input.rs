//! The input view: key to photon while typing into a focused input above a static list
//! (`typing`) and above a list that filters on every key (`search`), and the latency of the
//! filter itself.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use leptos::prelude::*;

use crate::state::RunView;
use crate::views::kit::{DotsCard, ScopePicker, ViewHead, WithData};

const BOTH: [(&str, &str); 2] = [
    ("typing", "static list"),
    ("search", "the list filters on every key"),
];

/// The typing metrics: section, metric, title, axis words.
const TYPING: [(&str, &str, &str); 6] = [
    ("latency_p50", "Key to photon, median", "milliseconds"),
    (
        "latency_p95",
        "Key to photon, 95th percentile",
        "milliseconds",
    ),
    ("latency_max", "Key to photon, slowest key", "milliseconds"),
    ("handling_p50", "Handling time, median", "milliseconds"),
    (
        "latency_slope",
        "Latency slope",
        "ms per key · above zero: the app falls behind",
    ),
    ("max_backlog", "Largest backlog", "keys sent but not shown"),
];

fn metrics() -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = TYPING
        .iter()
        .map(|(m, _, _)| MetricRef::new("typing", "typing", m))
        .collect();
    out.push(MetricRef::new("filter", "latency", "total_ms"));
    out.push(MetricRef::new("typing", "resources", "cpu_ms_per_s"));
    out
}

/// The input view.
#[component]
pub fn InputView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead
                title="Typing & search"
                text="A focused text input at 120 wpm above 1,000 or 10,000 rows. In search, every key filters the rows; z hides them all and BackSpace brings them back."
            />
            <RunView view={move |o: Arc<RunOverview>| view! {
                <WithData overview=o metrics=metrics() skeletons=4 view=move |data| {
                    let size = RwSignal::new(String::new());
                    let rate = RwSignal::new(String::new());
                    let d = data.clone();
                    view! {
                        <ScopePicker data=data.clone() scenarios=vec!["typing", "search"] size=size rate=rate/>
                        <div class="gx-grid">
                            <DotsCard
                                data=d.clone()
                                title="Filter latency"
                                words="z key to the emptied list, milliseconds"
                                metric=MetricRef::new("filter", "latency", "total_ms")
                                scenarios=vec![("search", "search")]
                                size=size
                                rate=rate
                            />
                            {TYPING.iter().map(|(m, title, words)| view! {
                                <DotsCard data=d.clone() title=*title words=*words metric=MetricRef::new("typing", "typing", m) scenarios=BOTH.to_vec() size=size rate=rate/>
                            }).collect_view()}
                            <DotsCard data=d.clone() title="App CPU while typing" words="CPU ms per second" metric=MetricRef::new("typing", "resources", "cpu_ms_per_s") scenarios=BOTH.to_vec() size=size rate=rate/>
                        </div>
                    }
                }/>
            }}/>
        </div>
    }
}
