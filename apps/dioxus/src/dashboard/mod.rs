//! The app scenario (`apps/shared/dashboard.rs`): a sidebar with the nav, a top bar with the
//! search, the live switch and the export menu, and a scroll container with the page of the
//! active nav item: the overview (the metric cards, the chart and filter cards, and a table of
//! `count` orders), the messages feed, or an empty page. The root owns the state as signals and
//! hands each part the signals it reads or writes; the stylesheet `dashboard.css` gives every
//! look, hover colors through `:hover`. The list scenario (`list.rs`) shows the orders alone.

mod chart;
mod filters;
mod kpis;
mod list;
mod live;
mod messages;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
pub mod spec;
mod switch;
mod topbar;

use dioxus::prelude::*;

use chart::Chart;
use filters::Filters;
use kpis::Kpis;
pub use list::OrderList;
use live::use_live;
use messages::Messages;
use orders::Orders;
use sidebar::Sidebar;
use spec::Page;
use topbar::Topbar;

/// The look of both scenarios.
const CSS: &str = include_str!("dashboard.css");

/// The state that the filter card resets: the search, the status filter, compare, the chart
/// series and the sorting of the orders.
#[derive(Clone, Copy, PartialEq)]
pub struct View {
    pub query: Signal<String>,
    /// `shown[k]` for `Status::ALL[k]`.
    pub shown: Signal<[bool; 4]>,
    pub compare: Signal<bool>,
    /// The index of the selected segment in `SEGMENTS`.
    pub series: Signal<usize>,
    /// Whether the orders are sorted by amount.
    pub sorted: Signal<bool>,
}

impl View {
    /// Checks every status, turns compare off, selects the first segment, clears the search and
    /// the sorting.
    pub fn reset(mut self) {
        self.shown.set([true; 4]);
        self.compare.set(false);
        self.series.set(0);
        self.query.set(String::new());
        self.sorted.set(false);
    }
}

/// Converts a spec color to CSS.
pub fn css_color(rgb: u32) -> String {
    format!("#{rgb:06x}")
}

/// The dashboard with `count` orders and `count` messages.
#[component]
pub fn Dashboard(count: usize) -> Element {
    let active = use_signal(|| 0usize);
    let view = View {
        query: use_signal(String::new),
        shown: use_signal(|| [true; 4]),
        compare: use_signal(|| false),
        series: use_signal(|| 0usize),
        sorted: use_signal(|| false),
    };
    let live = use_live();
    let page = use_memo(move || spec::page(active()));
    rsx! {
        style { {CSS} }
        div { class: "dash",
            Sidebar { active }
            div { class: "main",
                Topbar { query: view.query, active, live }
                div { class: "content",
                    match page() {
                        Page::Overview => rsx! {
                            div { class: "content__body",
                                Kpis { tick: live.tick }
                                div { class: "row2",
                                    Chart { series: view.series, compare: view.compare }
                                    Filters { view }
                                }
                                Orders { count, view }
                            }
                        },
                        Page::Messages => rsx! {
                            Messages { count }
                        },
                        Page::Empty => rsx! {
                            div { class: "content__body empty",
                                span { class: "empty__label", {spec::EMPTY_LABEL} }
                            }
                        },
                    }
                }
            }
        }
    }
}
