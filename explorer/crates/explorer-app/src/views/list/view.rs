//! The layout of the list view: the response to a hover and to a sort of the data, the slowest
//! frames and the missed refreshes of the scrolls, memory and startup, for every row count.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use explorer_core::story::BUDGETS;
use leptos::prelude::*;

use crate::state::{RunView, Selection, use_explorer};
use crate::views::kit::{
    LIST, NoCells, ScopeBar, ScopeDefault, ScopeDotsCard, StoryLegend, ViewHead, WithData, missed,
    response, slowest, use_scope,
};

/// The pacing and footprint cards: title, words of the axis, metric.
fn cards() -> Vec<(&'static str, &'static str, MetricRef)> {
    vec![
        (
            "Fling: slowest frames",
            "ms between two frames, 99th percentile",
            slowest("scroll-fling"),
        ),
        (
            "Fling: refreshes without a new frame",
            "% of the refreshes of the fling",
            missed("scroll-fling"),
        ),
        (
            "Glide: slowest frames",
            "ms between two frames, 99th percentile",
            slowest("scroll"),
        ),
        (
            "Glide: refreshes without a new frame",
            "% of the refreshes of the glide",
            missed("scroll"),
        ),
        (
            "Memory",
            "MiB, proportional set size at rest",
            MetricRef::new("idle", "resources", "pss_max"),
        ),
        (
            "Startup",
            "ms from launch to the frame with the content",
            MetricRef::new("startup", "startup", "content_ms"),
        ),
    ]
}

/// The response cards: title, words of the axis, measurement.
const RESPONSES: [(&str, &str, &str); 2] = [
    (
        "Hover a row",
        "ms from the pointer to the finished frame",
        "hover-row",
    ),
    (
        "Sort the data",
        "ms from the click to the finished frame",
        "click-sort",
    ),
];

/// Returns every metric the view reads.
fn metrics() -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = RESPONSES.iter().map(|(_, _, m)| response(m)).collect();
    out.extend(cards().into_iter().map(|(_, _, m)| m));
    out
}

/// The list view.
#[component]
pub fn ListView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead
                title="List"
                text="The order rows of the app in the framework's virtual list, with 10,000 and 1,000,000 rows of data, of which only the visible ones exist. A fling brings many new rows into view per frame. Hollow dots: the framework has no virtual list, so the app builds one by hand."
            />
            <RunView view={move |o: Arc<RunOverview>| view! { <Body overview=o/> }}/>
        </div>
    }
}

#[component]
fn Body(overview: Arc<RunOverview>) -> impl IntoView {
    let explorer = use_explorer();
    let scope = use_scope(
        overview.clone(),
        Signal::stored(LIST.to_owned()),
        ScopeDefault::MostCells,
    );
    let ov = overview.clone();
    let has_cells = Memo::new(move |_| {
        let prefs = explorer.prefs.snapshot();
        !Selection::new(&ov, &prefs, LIST).cells.is_empty()
    });
    let budgets: Vec<(f64, String)> = BUDGETS
        .iter()
        .map(|(hz, ms)| (*ms, format!("{hz} Hz frame")))
        .collect();
    move || {
        if !has_cells.get() {
            return view! { <NoCells scenario=LIST/> }.into_any();
        }
        let budgets = budgets.clone();
        view! {
            <div class="gx-view__controls"><ScopeBar scope=scope size=false/></div>
            <StoryLegend overview=overview.clone() scenario=LIST/>
            <WithData overview=overview.clone() metrics=metrics() skeletons=4 view=move |data| {
                view! {
                    <section class="gx-section">
                        <h2>"A long list" <span>"every row count at the refresh rate picked above · lower is better"</span></h2>
                        <div class="gx-grid gx-grid--narrow">
                            {RESPONSES.iter().map(|(title, words, m)| view! {
                                <ScopeDotsCard data=data.clone() title=*title words=words metric=response(m) scenario=LIST scope=scope every_size=true rules=budgets.clone()/>
                            }).collect_view()}
                            {cards().into_iter().map(|(title, words, metric)| view! {
                                <ScopeDotsCard data=data.clone() title=title words=words metric=metric scenario=LIST scope=scope every_size=true/>
                            }).collect_view()}
                        </div>
                    </section>
                }
            }/>
        }
        .into_any()
    }
}
