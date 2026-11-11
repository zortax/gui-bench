//! The views of a run, one module each, and the host that picks the view of the route.

mod cell;
mod compare;
mod explore;
mod footprint;
mod input;
mod kit;
mod overview;
mod provenance;
mod resize;
mod runs;
mod scaling;
mod steady;
mod table;

pub use cell::CellDrawer;
pub use provenance::Provenance;

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// The views of the explorer, in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Scaling,
    Table,
    Steady,
    Input,
    Resize,
    Footprint,
    Compare,
    Explore,
    Runs,
}

impl View {
    pub const ALL: [View; 10] = [
        View::Overview,
        View::Scaling,
        View::Table,
        View::Steady,
        View::Input,
        View::Resize,
        View::Footprint,
        View::Compare,
        View::Explore,
        View::Runs,
    ];

    /// Returns the path segment.
    pub fn slug(self) -> &'static str {
        match self {
            View::Overview => "overview",
            View::Scaling => "scaling",
            View::Table => "table",
            View::Steady => "steady",
            View::Input => "input",
            View::Resize => "resize",
            View::Footprint => "footprint",
            View::Compare => "compare",
            View::Explore => "explore",
            View::Runs => "runs",
        }
    }

    /// Returns the tab label.
    pub fn label(self) -> &'static str {
        match self {
            View::Overview => "Overview",
            View::Scaling => "Rows & scroll",
            View::Table => "Table ops",
            View::Steady => "Ticker & spinner",
            View::Input => "Typing & search",
            View::Resize => "Resize",
            View::Footprint => "Memory & idle",
            View::Compare => "Compare",
            View::Explore => "Explore",
            View::Runs => "Runs",
        }
    }

    /// Returns what the view shows, for its tab tooltip.
    pub fn description(self) -> &'static str {
        match self {
            View::Overview => "The run at a glance: health and a scorecard of every framework",
            View::Scaling => "Hover latency, scroll and startup against row count (rows, virtual)",
            View::Table => {
                "Keyed table operations: latency, CPU per operation, variant speedup, churn"
            }
            View::Steady => {
                "Steady animation: CPU and GPU time, frames and damage (ticker, spinner)"
            }
            View::Input => "Key to photon while typing, and filtering a list (typing, search)",
            View::Resize => "Resize storm and scale switch: configure to present",
            View::Footprint => "Memory and VRAM against rows, and what an idle app does",
            View::Compare => "One framework, variant or run against another, every metric",
            View::Explore => "Any metric against any other",
            View::Runs => "Every run with its provenance, and the differences of two runs",
        }
    }

    fn parse(slug: &str) -> Option<View> {
        View::ALL.into_iter().find(|v| v.slug() == slug)
    }
}

/// Renders the view of the route.
#[component]
pub fn ViewHost() -> impl IntoView {
    let params = use_params_map();
    let view = Memo::new(move |_| {
        params
            .with(|p| p.get("view"))
            .map_or(Some(View::Overview), |s| View::parse(&s))
    });
    move || match view.get() {
        Some(View::Overview) => overview::OverviewView().into_any(),
        Some(View::Scaling) => scaling::ScalingView().into_any(),
        Some(View::Table) => table::TableView().into_any(),
        Some(View::Steady) => steady::SteadyView().into_any(),
        Some(View::Input) => input::InputView().into_any(),
        Some(View::Resize) => resize::ResizeView().into_any(),
        Some(View::Footprint) => footprint::FootprintView().into_any(),
        Some(View::Compare) => compare::CompareView().into_any(),
        Some(View::Explore) => explore::ExploreView().into_any(),
        Some(View::Runs) => runs::RunsView().into_any(),
        None => view! { <p class="gx-notfound">"No such view."</p> }.into_any(),
    }
}
