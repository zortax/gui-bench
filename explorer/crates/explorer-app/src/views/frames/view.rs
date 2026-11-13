//! The layout of the frame-times view: the source and unit pickers, the frame percentiles
//! against size, and one size up close with its profile, late frames and gap classes.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use super::gaps::GapCards;
use super::profile::ProfileCard as Profile;
use super::scaling::FrameScaling;
use super::source::{SOURCES, Source, metrics};
use crate::state::{RunView, Selection, use_explorer, use_param};
use crate::views::kit::{ScopeBar, ScopeDefault, StoryLegend, ViewHead, WithData, use_scope};
use crate::widgets::{Empty, Segmented};

/// The frame-times view.
#[component]
pub fn FramesView() -> impl IntoView {
    let (unit, set_unit) = use_param("fu", "ms");
    let per_refresh = Signal::derive(move || unit.get() == "refresh");
    view! {
        <div class="gx-view">
            <ViewHead
                title="Frame times"
                text="Every measurement with continuous frames, of the app and of the list: how long each frame takes and where the screen repeats a stale frame. A frame that misses a refresh shows the last image again; the longest gaps are the stutters."
            >
                <Segmented
                    options=vec![("ms".to_owned(), "ms".to_owned()), ("refresh".to_owned(), "refreshes".to_owned())]
                    value=Signal::derive(move || unit.get())
                    on_change=set_unit
                    title="Frame times in milliseconds or in refresh periods"
                />
            </ViewHead>
            <RunView view={move |o: Arc<RunOverview>| view! { <Body overview=o per_refresh=per_refresh/> }}/>
        </div>
    }
}

#[component]
fn Body(overview: Arc<RunOverview>, per_refresh: Signal<bool>) -> impl IntoView {
    let explorer = use_explorer();
    let (key, set_key) = use_param("src", "app.scroll-fling");
    let present: Vec<Source> = SOURCES
        .iter()
        .copied()
        .filter(|s| s.present(&overview))
        .collect();
    let first = present
        .iter()
        .copied()
        .find(|s| s.measurement == "scroll-fling")
        .or_else(|| present.first().copied())
        .unwrap_or(SOURCES[0]);
    let options: Vec<(String, String)> = present
        .iter()
        .map(|s| (s.key(), s.label.to_owned()))
        .collect();
    let source = Memo::new(move |_| {
        let s = Source::parse(&key.get());
        if present.contains(&s) { s } else { first }
    });
    let scope = use_scope(
        overview.clone(),
        Signal::derive(move || source.get().scenario.to_owned()),
        ScopeDefault::MostCells,
    );
    let ov = overview.clone();
    view! {
        <div class="gx-view__controls">
            <Segmented
                options=options
                value=Signal::derive(move || source.get().key())
                on_change=set_key
                title="The measurement"
            />
        </div>
        {move || {
            let src = source.get();
            let prefs = explorer.prefs.snapshot();
            let sel = Selection::new(&ov, &prefs, src.scenario);
            if sel.cells.is_empty() {
                return view! {
                    <Empty
                        message=format!("No {} cells shown: the run has none, or the filters hide them.", src.scenario)
                        action="Show every framework"
                        on_action=Callback::new(move |_| explorer.query.set_all(vec![("hide", None), ("hz", None)]))
                    />
                }
                .into_any();
            }
            let ov2 = ov.clone();
            view! {
                <div class="gx-view__controls"><ScopeBar scope=scope size=false/></div>
                <StoryLegend overview=ov.clone() scenario=src.scenario/>
                <section class="gx-section">
                    <h2>"Against size" <span>{src.words}</span></h2>
                    <WithData overview=ov.clone() metrics=Signal::derive(move || metrics(&source.get())) skeletons=4 view=move |data| view! {
                        <div class="gx-grid">
                            <FrameScaling data=data.clone() source=src scope=scope metric="interval_p50" title="Typical frame · p50" words="ms per frame, median" per_refresh=per_refresh/>
                            <FrameScaling data=data.clone() source=src scope=scope metric="interval_p99" title="Slowest 1 % · p99" words="ms per frame, 99th percentile" per_refresh=per_refresh/>
                            <FrameScaling data=data.clone() source=src scope=scope metric="interval_max" title="Longest gap" words="ms between two presents, worst" per_refresh=per_refresh/>
                            <FrameScaling data=data.clone() source=src scope=scope metric="missed_share" title="Refreshes without a new frame" words="% of the refreshes of the measurement" per_refresh=per_refresh/>
                        </div>
                    }/>
                </section>
                <section class="gx-section">
                    <h2>"One size up close" <span>"every framework at one size and the refresh rate picked above"</span></h2>
                    <div class="gx-view__controls"><ScopeBar scope=scope rate=false/></div>
                    <WithData overview=ov2.clone() metrics=Signal::derive(move || metrics(&source.get())) skeletons=1 view=move |data| view! {
                        <div class="gx-grid gx-grid--wide">
                            <Profile data=data source=src scope=scope per_refresh=per_refresh/>
                        </div>
                    }/>
                    <div class="gx-grid gx-grid--wide">
                        <GapCards overview=ov2.clone() source=src scope=scope/>
                    </div>
                </section>
            }
            .into_any()
        }}
    }
}
