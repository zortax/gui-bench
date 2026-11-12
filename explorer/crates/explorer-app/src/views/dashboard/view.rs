//! The layout of the dashboard view: every interaction at a glance, each interaction from the
//! small to the large table, one interaction up close, the scroll glide frame by frame, resize,
//! startup and footprint, and the screenshots.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use leptos::prelude::*;

use super::gallery::Gallery;
use super::glance::{Glance, PROBES};
use super::growth::GrowthCard;
use super::stages::{self, StagesCard};
use crate::chart::{Legend, LegendItem};
use crate::state::{RunView, Selection, use_explorer, use_param};
use crate::views::frames::{GapCards, ProfileCard, SOURCES, Source};
use crate::views::kit::{ScopeBar, ViewHead, WithData, use_scope};
use crate::widgets::{Empty, Segmented};

/// The interactions: measurement, title, what happens.
const INTERACTIONS: [(&str, &str, &str); 4] = [
    (
        "hover-nav",
        "Hover a nav item",
        "the item takes the hover color",
    ),
    (
        "hover-row",
        "Hover an order row",
        "the row takes the hover color",
    ),
    (
        "click-compare",
        "Click the compare switch",
        "the chart redraws with the previous period",
    ),
    (
        "click-filter",
        "Uncheck a status",
        "the table drops the refunded orders",
    ),
];

/// Returns the options of the interaction picker.
fn interaction_options() -> Vec<(String, String)> {
    INTERACTIONS
        .iter()
        .map(|(m, t, _)| ((*m).to_owned(), (*t).to_owned()))
        .collect()
}

/// Returns every metric the view reads.
fn metrics(scroll: &Source) -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = PROBES
        .iter()
        .map(|(_, m, s, n)| MetricRef::new(m, s, n))
        .collect();
    let names: Vec<&str> = INTERACTIONS.iter().map(|(m, ..)| *m).collect();
    out.extend(stages::metrics(&names));
    out.extend(
        names
            .iter()
            .map(|m| MetricRef::new(m, "frame_cost", "cpu_ms_per_frame")),
    );
    out.extend(crate::views::frames::metrics(scroll));
    for (m, s, n) in [
        ("scroll", "scroll_smoothness", "smoothness_score"),
        ("resize", "resize", "configure_to_present_p50"),
        ("resize", "frame_pacing", "interval_max"),
        ("resize", "resize", "mismatched_frames"),
        ("scale", "resize", "configure_to_present_p50"),
        ("startup", "startup", "first_present_ms"),
        ("idle", "resources", "pss_max"),
        ("idle", "resources", "gpu_proc_mem_max"),
        ("idle", "resources", "cpu_ms_per_s"),
    ] {
        out.push(MetricRef::new(m, s, n));
    }
    out
}

/// The dashboard view.
#[component]
pub fn DashboardView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead
                title="Dashboard"
                text="An analytics app as people build them: a sidebar, cards with shadows, a bar chart, controls and an orders table of 50 or 1,000 rows, every row a real element. How fast it answers, how it scrolls and resizes, and what it costs."
            />
            <RunView view={move |o: Arc<RunOverview>| view! { <Body overview=o/> }}/>
        </div>
    }
}

#[component]
fn Body(overview: Arc<RunOverview>) -> impl IntoView {
    let explorer = use_explorer();
    let scope = use_scope(overview.clone(), Signal::stored("dashboard".to_owned()));
    let (pick, set_pick) = use_param("ix", "click-filter");
    let interaction = Signal::derive(move || {
        let want = pick.get();
        INTERACTIONS
            .iter()
            .find(|(m, ..)| *m == want)
            .map_or("click-filter", |(m, ..)| *m)
    });
    let scroll = SOURCES
        .iter()
        .copied()
        .find(|s| s.scenario == "dashboard")
        .unwrap_or(SOURCES[0]);
    let hz = Signal::derive(move || scope.hz.get());
    let ov = overview.clone();
    let has_cells = move || {
        let prefs = explorer.prefs.snapshot();
        !Selection::new(&ov, &prefs, "dashboard").cells.is_empty()
    };
    let ov = overview.clone();
    let legend = move || {
        let prefs = explorer.prefs.snapshot();
        Selection::new(&ov, &prefs, "dashboard")
            .frameworks()
            .iter()
            .map(|f| LegendItem::framework(f))
            .collect::<Vec<_>>()
    };
    let ov = overview.clone();
    let content = move || {
        if !has_cells() {
            return view! {
                <Empty
                    message="No dashboard cells shown: the run has none, or the filters hide them."
                    action="Show every framework"
                    on_action=Callback::new(move |_| explorer.query.set_all(vec![("hide", None), ("var", None), ("hz", None)]))
                />
            }
            .into_any();
        }
        let ov = ov.clone();
        let overview = overview.clone();
        let legend = legend.clone();
        view! {
            <div class="gx-view__controls"><ScopeBar scope=scope noun="orders"/></div>
            <div class="gx-legendbar"><Legend items=Signal::derive(legend)/></div>
            <WithData overview=ov.clone() metrics=metrics(&scroll) skeletons=4 view=move |data| {
                let ov = ov.clone();
                let cpu_data = data.clone();
                view! {
                    <section class="gx-section">
                        <h2>"At a glance" <span>"the refresh rate picked above"</span></h2>
                        <Glance data=data.clone() hz=hz/>
                    </section>
                    <section class="gx-section">
                        <h2>"Input to photon" <span>"from the small to the large table, every repeat of every iteration"</span></h2>
                        <div class="gx-grid">
                            {INTERACTIONS.iter().map(|(m, title, words)| view! {
                                <GrowthCard
                                    data=data.clone()
                                    scenario="dashboard"
                                    metric=MetricRef::new(m, "latency", "total_ms")
                                    title=*title
                                    words=words
                                    hz=hz
                                    refresh_rules=true
                                />
                            }).collect_view()}
                        </div>
                    </section>
                    <section class="gx-section">
                        <h2>"One interaction up close" <span>"at the size and rate picked above"</span></h2>
                        <div class="gx-view__controls">
                            <Segmented
                                options=interaction_options()
                                value=Signal::derive(move || interaction.get().to_owned())
                                on_change=set_pick
                                title="The interaction"
                            />
                        </div>
                        <div class="gx-grid gx-grid--wide">
                            <StagesCard data=data.clone() interaction=interaction scope=scope/>
                        </div>
                        <div class="gx-grid">
                            {move || {
                                let m = interaction.get();
                                view! {
                                    <GrowthCard
                                        data=cpu_data.clone()
                                        scenario="dashboard"
                                        metric=MetricRef::new(m, "frame_cost", "cpu_ms_per_frame")
                                        title="App CPU per frame"
                                        words="CPU ms per presented frame"
                                        hz=hz
                                        refresh_rules=true
                                    />
                                }
                            }}
                            <GrowthCard
                                data=data.clone()
                                scenario="dashboard"
                                metric=MetricRef::new("scroll", "scroll_smoothness", "smoothness_score")
                                title="Scroll smoothness"
                                words="score, 0 to 100 · higher is better"
                                hz=hz
                            />
                        </div>
                    </section>
                    <section class="gx-section">
                        <h2>"Scrolling the dashboard" <span>{scroll.words}</span></h2>
                        <div class="gx-grid gx-grid--wide">
                            <ProfileCard data=data.clone() source=scroll scope=scope per_refresh=Signal::stored(false) title="Scroll frame times"/>
                            <GapCards overview=ov.clone() source=scroll scope=scope/>
                        </div>
                    </section>
                    <section class="gx-section">
                        <h2>"Resizing the dashboard" <span>"a resize storm and a switch to scale 2, 60 Hz"</span></h2>
                        <div class="gx-grid">
                            <GrowthCard data=data.clone() scenario="dashboard-resize" metric=MetricRef::new("resize", "resize", "configure_to_present_p50") title="Resize: configure to present" words="ms, median configure" hz=hz refresh_rules=true/>
                            <GrowthCard data=data.clone() scenario="dashboard-resize" metric=MetricRef::new("resize", "frame_pacing", "interval_max") title="Resize: longest frame gap" words="ms between two presents, worst" hz=hz refresh_rules=true/>
                            <GrowthCard data=data.clone() scenario="dashboard-resize" metric=MetricRef::new("resize", "resize", "mismatched_frames") title="Resize: frames at a stale size" words="frames whose buffer misses the configured size" hz=hz/>
                            <GrowthCard data=data.clone() scenario="dashboard-resize" metric=MetricRef::new("scale", "resize", "configure_to_present_p50") title="Scale switch: configure to present" words="ms, median switch" hz=hz refresh_rules=true/>
                        </div>
                    </section>
                    <section class="gx-section">
                        <h2>"Startup and footprint" <span>"spawn to the first frame, and the idle app after it"</span></h2>
                        <div class="gx-grid">
                            <GrowthCard data=data.clone() scenario="dashboard" metric=MetricRef::new("startup", "startup", "first_present_ms") title="First frame" words="ms from spawn" hz=hz/>
                            <GrowthCard data=data.clone() scenario="dashboard" metric=MetricRef::new("idle", "resources", "pss_max") title="Memory (PSS)" words="MiB, idle" hz=hz/>
                            <GrowthCard data=data.clone() scenario="dashboard" metric=MetricRef::new("idle", "resources", "gpu_proc_mem_max") title="VRAM" words="MiB, idle" hz=hz/>
                            <GrowthCard data=data.clone() scenario="dashboard" metric=MetricRef::new("idle", "resources", "cpu_ms_per_s") title="Idle CPU" words="CPU ms per second" hz=hz/>
                        </div>
                    </section>
                }
            }/>
            <section class="gx-section">
                <h2>"How each framework draws it" <span>"the screenshot after startup, at the size and rate picked above"</span></h2>
                <Gallery overview=overview scope=scope/>
            </section>
        }
        .into_any()
    };
    view! { {content} }
}
