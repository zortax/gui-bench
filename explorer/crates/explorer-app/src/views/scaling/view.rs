//! The layout of the rows view: the scenario switch, the legend, and the chart sections.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use leptos::prelude::*;

use super::charts::{Distribution, LatencyVsRows, ScrollMetric, StartupWaterfall};
use crate::chart::{Legend, LegendItem};
use crate::palette::{Marker, hz_dash};
use crate::state::{RunView, Selection, use_explorer, use_param};
use crate::views::frames::{FrameScaling, SOURCES, Source};
use crate::views::kit::{ViewHead, WithData};
use crate::widgets::{Empty, Segmented};

/// The metrics the rows view reads.
fn metrics() -> Vec<MetricRef> {
    [
        ("hover", "latency", "total_ms"),
        ("hover", "latency", "input_to_commit_p50"),
        ("scroll", "scroll_smoothness", "smoothness_score"),
        ("scroll", "frame_pacing", "interval_p99"),
        ("scroll", "frame_pacing", "interval_max"),
        ("scroll", "scroll_smoothness", "missed_vblanks"),
        ("scroll", "scroll_smoothness", "tracking_ratio"),
        ("scroll", "resources", "cpu_ms_per_s"),
        ("startup", "startup", "first_present_ms"),
        ("startup", "startup", "connected_ms"),
        ("startup", "startup", "first_surface_ms"),
        ("startup", "startup", "toplevel_ms"),
        ("startup", "startup", "first_buffer_ms"),
    ]
    .iter()
    .map(|(a, b, c)| MetricRef::new(a, b, c))
    .collect()
}

/// Returns the frame source of the scroll glide of a scenario.
fn scroll_source(scenario: &str) -> Source {
    SOURCES
        .iter()
        .copied()
        .find(|s| s.scenario == scenario && s.measurement == "scroll")
        .unwrap_or(SOURCES[0])
}

/// The rows view.
#[component]
pub fn ScalingView() -> impl IntoView {
    let (scenario, set_scenario) = use_param("sc", "rows");
    let options = vec![
        ("rows".to_owned(), "all rows built".to_owned()),
        ("virtual".to_owned(), "virtualized list".to_owned()),
    ];
    view! {
        <div class="gx-view">
            <ViewHead
                title="Rows & scroll"
                text="Input to photon of a hover, scroll glides and startup, against the number of rows the app holds."
            >
                <Segmented options=options value=Signal::derive(move || scenario.get()) on_change=set_scenario/>
            </ViewHead>
            <RunView view={move |o: Arc<RunOverview>| view! { <Body overview=o scenario=scenario/> }}/>
        </div>
    }
}

#[component]
fn Body(overview: Arc<RunOverview>, scenario: Memo<String>) -> impl IntoView {
    let explorer = use_explorer();
    let ov = overview.clone();
    move || {
        let sc = scenario.get();
        let prefs = explorer.prefs.snapshot();
        let sel = Selection::new(&ov, &prefs, &sc);
        if sel.cells.is_empty() {
            return view! {
                <Empty
                    message=format!("No {sc} cells shown: the run has none, or the filters hide them.")
                    action="Show every framework"
                    on_action=Callback::new(move |_| explorer.query.set_all(vec![("hide", None), ("var", None), ("hz", None)]))
                />
            }
            .into_any();
        }
        let rates = sel.rates();
        let mut legend: Vec<LegendItem> = sel
            .frameworks()
            .iter()
            .map(|f| LegendItem::framework(f))
            .collect();
        if rates.len() > 1 {
            legend.extend(rates.iter().map(|hz| {
                LegendItem::encoding(
                    format!("{hz} Hz"),
                    Marker::Circle,
                    Some(hz_dash(*hz).to_owned()),
                    false,
                )
            }));
        }
        let ov = ov.clone();
        let sc2 = sc.clone();
        view! {
            <WithData overview=ov metrics=Signal::derive(metrics) skeletons=4 view=move |data| {
                let sc = sc2.clone();
                let legend = legend.clone();
                let rates = rates.clone();
                view! {
                    <section class="gx-section">
                        <h2>"Hover latency" <span>"input to the first frame that shows the hover color"</span></h2>
                        <div class="gx-legendbar"><Legend items=legend.clone()/></div>
                        <div class="gx-grid gx-grid--wide">
                            {rates.iter().map(|hz| view! { <LatencyVsRows data=data.clone() scenario=sc.clone() hz=*hz/> }).collect_view()}
                        </div>
                        <div class="gx-grid gx-grid--wide">
                            <Distribution data=data.clone() scenario=sc.clone()/>
                            <ScrollMetric
                                data=data.clone()
                                scenario=sc.clone()
                                title="App share of the hover latency"
                                metric=MetricRef::new("hover", "latency", "input_to_commit_p50")
                                words="input to commit, median of iterations"
                            />
                        </div>
                    </section>
                    <section class="gx-section">
                        <h2>"Scroll glide" <span>"1 s at 800 px/s, touchpad without kinetic scrolling · every frame in Frame times"</span></h2>
                        <div class="gx-grid">
                            <ScrollMetric data=data.clone() scenario=sc.clone() title="Smoothness" metric=MetricRef::new("scroll", "scroll_smoothness", "smoothness_score") words="score, 0 to 100 · higher is better"/>
                            <FrameScaling data=data.clone() source=scroll_source(&sc) metric="interval_p99" title="Slowest 1 % of frames · p99" words="ms per frame, 99th percentile" per_refresh=Signal::stored(false)/>
                            <FrameScaling data=data.clone() source=scroll_source(&sc) metric="interval_max" title="Longest gap" words="ms between two presents, worst" per_refresh=Signal::stored(false)/>
                            <ScrollMetric data=data.clone() scenario=sc.clone() title="Missed refreshes" metric=MetricRef::new("scroll", "scroll_smoothness", "missed_vblanks") words="per 1 s glide · lower is better"/>
                            <ScrollMetric data=data.clone() scenario=sc.clone() title="Tracking" metric=MetricRef::new("scroll", "scroll_smoothness", "tracking_ratio") words="content distance / input distance"/>
                            <ScrollMetric data=data.clone() scenario=sc.clone() title="App CPU while scrolling" metric=MetricRef::new("scroll", "resources", "cpu_ms_per_s") words="CPU ms per second"/>
                        </div>
                    </section>
                    <section class="gx-section">
                        <h2>"Startup" <span>"spawn to the first presented frame"</span></h2>
                        <div class="gx-grid gx-grid--wide">
                            <ScrollMetric data=data.clone() scenario=sc.clone() title="First presented frame" metric=MetricRef::new("startup", "startup", "first_present_ms") words="ms from spawn"/>
                            <StartupWaterfall data=data.clone() scenario=sc.clone()/>
                        </div>
                    </section>
                }
            }/>
        }
        .into_any()
    }
}
