//! The layout of the overview.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use super::health::Health;
use super::scorecard::Scorecard;
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
                text="The run at a glance: where it comes from, how healthy its reports are, and how every framework fares on the key metric of each scenario."
            />
            <RunView view={move |o: Arc<RunOverview>| view! {
                <Scorecard overview=o.clone()/>
                <div class="gx-grid gx-grid--wide">
                    <section class="gx-card">
                        <h3 class="gx-chart__title">"Provenance"</h3>
                        <Provenance overview=o.clone()/>
                    </section>
                    <Health overview=o/>
                </div>
            }}/>
        </div>
    }
}
