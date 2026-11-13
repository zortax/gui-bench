//! The layout of the overview.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use super::builds::Builds;
use super::capacity::CapacityCards;
use super::health::Health;
use crate::state::RunView;
use crate::views::Provenance;
use crate::views::kit::ViewHead;

/// The overview view.
#[component]
pub fn OverviewView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead
                title="Overview"
                text="The run at a glance: how large the app can grow before an interaction misses a 144 Hz or a 60 Hz frame, where the run comes from, how healthy its reports are, and what each app builds by hand."
            />
            <RunView view={move |o: Arc<RunOverview>| view! {
                <CapacityCards overview=o.clone()/>
                <div class="gx-grid gx-grid--wide">
                    <section class="gx-card">
                        <h3 class="gx-chart__title">"Provenance"</h3>
                        <Provenance overview=o.clone()/>
                    </section>
                    <Health overview=o.clone()/>
                    <Builds overview=o/>
                </div>
            }}/>
        </div>
    }
}
