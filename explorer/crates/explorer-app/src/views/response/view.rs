//! The layout of the response view: the legend, one chart per interaction on a shared scale,
//! and the table of slopes.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use explorer_core::story::RESPONSES;
use leptos::prelude::*;

use super::charts::{ResponseCard, shared_top};
use super::slope::SlopeCard;
use crate::state::{RunView, Selection, use_explorer, use_param};
use crate::views::kit::{APP, NoCells, StoryLegend, ViewHead, WithData, response_metrics};
use crate::widgets::Toggle;

/// The response view.
#[component]
pub fn ResponseView() -> impl IntoView {
    let (share, set_share) = use_param("share", "1");
    let shared = Signal::derive(move || share.get() == "1");
    view! {
        <div class="gx-view">
            <ViewHead
                title="Response time"
                text="The time from an input to the commit of the frame that shows the response, plus the time until the GPU finished it, without the wait for the next refresh. A flat line is work that follows the change; a rising line is work that follows the size of the app."
            >
                <Toggle
                    label="one scale for every chart"
                    value=shared
                    on_change=Callback::new(move |on: bool| set_share.run(if on { "1" } else { "0" }.to_owned()))
                    title="Every chart takes the same value axis, so a cheap interaction sits low and a costly one high"
                />
            </ViewHead>
            <RunView view={move |o: Arc<RunOverview>| view! { <Body overview=o shared=shared/> }}/>
        </div>
    }
}

#[component]
fn Body(overview: Arc<RunOverview>, shared: Signal<bool>) -> impl IntoView {
    let explorer = use_explorer();
    let ov = overview.clone();
    let has_cells = Memo::new(move |_| {
        let prefs = explorer.prefs.snapshot();
        !Selection::new(&ov, &prefs, APP).cells.is_empty()
    });
    move || {
        if !has_cells.get() {
            return view! { <NoCells scenario=APP/> }.into_any();
        }
        view! {
            <StoryLegend overview=overview.clone() scenario=APP stops=true/>
            <WithData overview=overview.clone() metrics=response_metrics() skeletons=4 view=move |data| {
                let d = data.clone();
                let top = Signal::derive(move || shared.get().then(|| shared_top(&d, &explorer.prefs.snapshot())).flatten());
                view! {
                    <section class="gx-section">
                        <h2>"As the app grows" <span>"every repeat of every iteration, at the refresh rate of every cell · lower is better"</span></h2>
                        <div class="gx-grid gx-grid--narrow">
                            {RESPONSES.iter().map(|step| view! { <ResponseCard data=data.clone() step=*step top=top/> }).collect_view()}
                        </div>
                    </section>
                    <section class="gx-section">
                        <h2>"What a larger app costs" <span>"milliseconds of response time per 1,000 elements more"</span></h2>
                        <div class="gx-grid gx-grid--wide">
                            <SlopeCard data=data.clone()/>
                        </div>
                    </section>
                }
            }/>
        }
        .into_any()
    }
}
