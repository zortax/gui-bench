//! The layout of the smoothness view: the rate picker, the legend, the pacing measurements and
//! the resize against the size of the app, and the fling frame by frame.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use explorer_core::story::PACED;
use leptos::prelude::*;

use super::charts::{Judge, PacingCard};
use crate::state::{RunView, Selection, use_explorer};
use crate::views::frames::{GapCards, Source, metrics as frame_metrics};
use crate::views::kit::{
    APP, NoCells, ScopeBar, ScopeDefault, StoryLegend, ViewHead, WithData, missed, slowest,
    use_scope,
};

/// The measurement of the frame-by-frame section.
const FLING: &str = "scroll-fling";

fn resize(metric: &str) -> MetricRef {
    MetricRef::new("resize", "resize", metric)
}

/// Returns every metric the view reads.
fn metrics() -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = PACED
        .iter()
        .flat_map(|s| [slowest(s.name), missed(s.name)])
        .collect();
    out.push(resize("configure_to_present_p50"));
    out.push(resize("mismatched_frames"));
    out.extend(frame_metrics(&Source::of(APP, FLING)));
    out
}

/// The smoothness view.
#[component]
pub fn SmoothnessView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead
                title="Smoothness"
                text="The continuous interactions of the app as it grows: the slowest frames and the share of refreshes without a new frame while scrolling and animating, and how long a resize takes to reach the screen. A cell that misses more than 1 % of the refreshes runs these steps again at the slower rate."
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
        ScopeDefault::MostCells,
    );
    let ov = overview.clone();
    let has_cells = Memo::new(move |_| {
        let prefs = explorer.prefs.snapshot();
        !Selection::new(&ov, &prefs, APP).cells.is_empty()
    });
    move || {
        if !has_cells.get() {
            return view! { <NoCells scenario=APP/> }.into_any();
        }
        let ov = overview.clone();
        view! {
            <div class="gx-view__controls"><ScopeBar scope=scope size=false/></div>
            <StoryLegend overview=overview.clone() scenario=APP/>
            <WithData overview=overview.clone() metrics=metrics() skeletons=4 view=move |data| {
                view! {
                    <section class="gx-section">
                        <h2>"As the app grows" <span>"median iteration at the refresh rate picked above · lower is better"</span></h2>
                        <div class="gx-grid">
                            {PACED.iter().map(|step| view! {
                                <PacingCard
                                    data=data.clone()
                                    title=format!("{}: slowest frames", step.title)
                                    metric=slowest(step.name)
                                    words="ms between frames, 99th percentile"
                                    judge=Judge::Refresh
                                    scope=scope
                                />
                                <PacingCard
                                    data=data.clone()
                                    title=format!("{}: refreshes without a new frame", step.title)
                                    metric=missed(step.name)
                                    words="% of the refreshes of the measurement"
                                    judge=Judge::Smooth
                                    scope=scope
                                />
                            }).collect_view()}
                            <PacingCard
                                data=data.clone()
                                title="Resize: new size to the screen"
                                metric=resize("configure_to_present_p50")
                                words="ms from the new size to its frame, median"
                                judge=Judge::Refresh
                                scope=scope
                            />
                            <PacingCard
                                data=data.clone()
                                title="Resize: frames of an outdated size"
                                metric=resize("mismatched_frames")
                                words="outdated frames per storm"
                                judge=Judge::Count
                                scope=scope
                            />
                        </div>
                    </section>
                }
            }/>
            <section class="gx-section">
                <h2>"A fling, frame by frame" <span>"every late frame of every iteration at one size · the size most frameworks flung at, unless picked"</span></h2>
                <div class="gx-view__controls"><ScopeBar scope=scope rate=false/></div>
                <div class="gx-grid gx-grid--wide">
                    <GapCards overview=ov source=Source::of(APP, FLING) scope=scope/>
                </div>
            </section>
        }
        .into_any()
    }
}
