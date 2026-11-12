//! The dashboard scenario (`apps/shared/dashboard.rs`): a sidebar, a top bar, and a scroll
//! container with the metric cards, the chart and filter cards, and a table of `count` orders.
//! The root owns the state as signals and hands each part the signals it reads or writes; the
//! stylesheet `dashboard.css` gives every look, hover colors through `:hover`.

mod chart;
mod filters;
mod kpis;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
mod spec;
mod topbar;

use dioxus::prelude::*;

use chart::Chart;
use filters::Filters;
use kpis::Kpis;
use orders::Orders;
use sidebar::Sidebar;
use topbar::Topbar;

/// The look of the whole dashboard.
const CSS: &str = include_str!("dashboard.css");

/// The state that the filter card resets: the search, the status filter, compare and the chart
/// series.
#[derive(Clone, Copy, PartialEq)]
pub struct View {
    pub query: Signal<String>,
    /// `shown[k]` for `Status::ALL[k]`.
    pub shown: Signal<[bool; 4]>,
    pub compare: Signal<bool>,
    /// The index of the selected segment in `SEGMENTS`.
    pub series: Signal<usize>,
}

impl View {
    /// Checks every status, turns compare off, selects the first segment and clears the search.
    pub fn reset(mut self) {
        self.shown.set([true; 4]);
        self.compare.set(false);
        self.series.set(0);
        self.query.set(String::new());
    }
}

/// Converts a spec color to CSS.
pub fn css_color(rgb: u32) -> String {
    format!("#{rgb:06x}")
}

/// The dashboard with `count` orders.
#[component]
pub fn Dashboard(count: usize) -> Element {
    let active = use_signal(|| 0usize);
    let view = View {
        query: use_signal(String::new),
        shown: use_signal(|| [true; 4]),
        compare: use_signal(|| false),
        series: use_signal(|| 0usize),
    };
    rsx! {
        style { {CSS} }
        div { class: "dash",
            Sidebar { active }
            div { class: "main",
                Topbar { query: view.query }
                div { class: "content",
                    Kpis {}
                    div { class: "row2",
                        Chart { series: view.series, compare: view.compare }
                        Filters { view }
                    }
                    Orders { count, query: view.query, shown: view.shown }
                }
            }
        }
    }
}
