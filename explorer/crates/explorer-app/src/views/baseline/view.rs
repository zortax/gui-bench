//! The layout of the baseline view: the size picker, the cost of the app at that size, and the
//! screenshots.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use explorer_core::story::BUDGETS;
use leptos::prelude::*;

use super::binary::BinaryCard;
use super::gallery::Gallery;
use crate::state::{RunView, Selection, use_explorer};
use crate::views::kit::{
    APP, NoCells, ScopeBar, ScopeDefault, ScopeDotsCard, StoryLegend, ViewHead, WithData, response,
    size_words, use_scope,
};

/// The dot cards: title, words of the axis, metric.
const CARDS: [(&str, &str, (&str, &str, &str)); 7] = [
    (
        "Startup",
        "ms from launch to the frame with the content",
        ("startup", "startup", "content_ms"),
    ),
    (
        "Memory",
        "MiB, proportional set size at rest",
        ("idle", "resources", "pss_max"),
    ),
    (
        "Work at rest",
        "CPU ms per second with no input",
        ("idle", "resources", "cpu_ms_per_s"),
    ),
    (
        "First frame of any content",
        "ms from launch to the first presented frame",
        ("startup", "startup", "first_present_ms"),
    ),
    (
        "VRAM",
        "MiB of GPU memory of the app at rest",
        ("idle", "resources", "gpu_proc_mem_max"),
    ),
    (
        "Wakeups at rest",
        "context switches per second with no input",
        ("idle", "resources", "wakeups_per_s"),
    ),
    (
        "Private memory",
        "MiB, memory of the app alone at rest",
        ("idle", "resources", "private_max"),
    ),
];

/// Returns every metric the view reads.
fn metrics() -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = CARDS
        .iter()
        .map(|(_, _, (m, s, n))| MetricRef::new(m, s, n))
        .collect();
    out.push(response("hover-row"));
    out
}

/// The baseline view.
#[component]
pub fn BaselineView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead
                title="Baseline"
                text="What a small app costs: the time from launch to the frame with the content, memory and work at rest, the response to a hover, and the size of the binary. The smallest size of the app, unless another one is picked."
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
        Signal::stored(APP.to_owned()),
        ScopeDefault::Smallest,
    );
    let ov = overview.clone();
    let has_cells = Memo::new(move |_| {
        let prefs = explorer.prefs.snapshot();
        !Selection::new(&ov, &prefs, APP).cells.is_empty()
    });
    let budgets: Vec<(f64, String)> = BUDGETS
        .iter()
        .map(|(hz, ms)| (*ms, format!("{hz} Hz frame")))
        .collect();
    move || {
        if !has_cells.get() {
            return view! { <NoCells scenario=APP/> }.into_any();
        }
        let budgets = budgets.clone();
        let o_binary = overview.clone();
        view! {
            <div class="gx-view__controls">
                <ScopeBar scope=scope rate=false/>
                <span class="gx-muted">{move || scope.size.get().map(size_words)}</span>
            </div>
            <StoryLegend overview=overview.clone() scenario=APP/>
            <WithData overview=overview.clone() metrics=metrics() skeletons=4 view=move |data| {
                let card = |i: usize| {
                    let (title, words, (m, s, n)) = CARDS[i];
                    view! { <ScopeDotsCard data=data.clone() title=title words=words metric=MetricRef::new(m, s, n) scenario=APP scope=scope/> }
                };
                view! {
                    <section class="gx-section">
                        <h2>"The cost of a small app" <span>"at the refresh rate of every cell · lower is better"</span></h2>
                        <div class="gx-grid gx-grid--narrow">
                            {card(0)}
                            {card(1)}
                            {card(2)}
                            <ScopeDotsCard
                                data=data.clone()
                                title="Hover response"
                                words="ms from the pointer to the finished frame"
                                metric=response("hover-row")
                                scenario=APP
                                scope=scope
                                rules=budgets.clone()
                            />
                            <BinaryCard overview=o_binary.clone()/>
                        </div>
                    </section>
                    <section class="gx-section">
                        <h2>"More of the app at rest" <span>"the first frame, GPU memory, wakeups and private memory"</span></h2>
                        <div class="gx-grid gx-grid--narrow">
                            {card(3)}
                            {card(4)}
                            {card(5)}
                            {card(6)}
                        </div>
                    </section>
                }
            }/>
            <section class="gx-section">
                <h2>"How each framework draws it" <span>"the screenshot of the layout check, at the size picked above"</span></h2>
                <Gallery overview=overview.clone() scope=scope/>
            </section>
        }
        .into_any()
    }
}
