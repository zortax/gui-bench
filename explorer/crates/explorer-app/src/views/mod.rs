//! The views of a run, one module each, and the host that picks the view of the route.

mod baseline;
mod cell;
mod compare;
mod explore;
mod frames;
mod kit;
mod list;
mod overview;
mod provenance;
mod response;
mod runs;
mod smoothness;

pub use cell::CellDrawer;
pub use provenance::Provenance;

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// The views of the explorer, in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Response,
    Smoothness,
    Frames,
    Baseline,
    List,
    Compare,
    Explore,
    Runs,
}

impl View {
    pub const ALL: [View; 9] = [
        View::Overview,
        View::Response,
        View::Smoothness,
        View::Frames,
        View::Baseline,
        View::List,
        View::Compare,
        View::Explore,
        View::Runs,
    ];

    /// Returns the path segment.
    pub fn slug(self) -> &'static str {
        match self {
            View::Overview => "overview",
            View::Response => "response",
            View::Smoothness => "smoothness",
            View::Frames => "frames",
            View::Baseline => "baseline",
            View::List => "list",
            View::Compare => "compare",
            View::Explore => "explore",
            View::Runs => "runs",
        }
    }

    /// Returns the tab label.
    pub fn label(self) -> &'static str {
        match self {
            View::Overview => "Overview",
            View::Response => "Response time",
            View::Smoothness => "Smoothness",
            View::Frames => "Frame times",
            View::Baseline => "Baseline",
            View::List => "List",
            View::Compare => "Compare",
            View::Explore => "Explore",
            View::Runs => "Runs",
        }
    }

    /// Returns what the view shows, for its tab tooltip.
    pub fn description(self) -> &'static str {
        match self {
            View::Overview => {
                "The run at a glance: how far each framework scales, health, provenance and builds"
            }
            View::Response => {
                "The response time of every interaction as the app grows, and the cost per 1,000 elements"
            }
            View::Smoothness => {
                "Slowest frames and missed refreshes of scrolling, animation and resize as the app grows, and the fling frame by frame"
            }
            View::Frames => {
                "Frame-time percentiles and every late frame of any scroll, animation or resize"
            }
            View::Baseline => {
                "What a small app costs: startup, memory, work at rest, hover response, binary size"
            }
            View::List => "The virtual list at 10,000 and 1,000,000 rows",
            View::Compare => "One framework or run against another, every metric",
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
        Some(View::Response) => response::ResponseView().into_any(),
        Some(View::Smoothness) => smoothness::SmoothnessView().into_any(),
        Some(View::Frames) => frames::FramesView().into_any(),
        Some(View::Baseline) => baseline::BaselineView().into_any(),
        Some(View::List) => list::ListView().into_any(),
        Some(View::Compare) => compare::CompareView().into_any(),
        Some(View::Explore) => explore::ExploreView().into_any(),
        Some(View::Runs) => runs::RunsView().into_any(),
        None => view! { <p class="gx-notfound">"No such view."</p> }.into_any(),
    }
}
