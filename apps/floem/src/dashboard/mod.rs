//! The app scenario: a sidebar with nav, a top bar with the search, the live switch and the
//! export menu, and one scroll container with the page of the active nav item: the overview (the
//! metric cards, the chart and filter cards and the orders table), the messages feed, or an empty
//! page (`apps/shared/dashboard.rs`). The state lives in floem signals that the views read in
//! their closures; the scroll container scrolls as floem's does by default.

mod chart;
mod filters;
mod kpis;
pub mod list;
mod live;
mod messages;
mod orders;
mod sidebar;
mod style;
mod topbar;

use floem::prelude::*;
use floem::reactive::Memo;
use floem::taffy::Overflow;

use crate::dash::{self, Page};
use live::Live;
use style::{Weight, px, text};

/// The width floem's scroll container keeps for its vertical bar on the right of the content; the
/// bar sits in the right padding of the content, so the cards keep their spec positions.
const SCROLLBAR_WIDTH: f32 = 8.0;

/// The state of the controls, shared by the parts that read or write it.
#[derive(Clone, Copy)]
pub struct State {
    /// The active nav item.
    nav: RwSignal<usize>,
    /// The text of the search input.
    search: RwSignal<String>,
    /// The chart series, an index of `SEGMENTS`.
    series: RwSignal<usize>,
    /// The compare switch.
    compare: RwSignal<bool>,
    /// The status checkboxes, in the order of `Status::ALL`.
    shown: [RwSignal<bool>; 4],
    /// Whether the orders are sorted by amount.
    sorted: RwSignal<bool>,
    /// The live updates.
    live: Live,
}

impl State {
    fn new() -> Self {
        Self {
            nav: RwSignal::new(0),
            search: RwSignal::new(String::new()),
            series: RwSignal::new(0),
            compare: RwSignal::new(false),
            shown: std::array::from_fn(|_| RwSignal::new(true)),
            sorted: RwSignal::new(false),
            live: Live::new(),
        }
    }

    /// Returns the status filter, and makes the caller track it.
    fn shown(self) -> [bool; 4] {
        self.shown.map(|shown| shown.get())
    }

    /// The Reset button: every status checked, compare off, the first series, no search, no
    /// sorting.
    fn reset(self) {
        for shown in self.shown {
            shown.set(true);
        }
        self.compare.set(false);
        self.series.set(0);
        self.search.set(String::new());
        self.sorted.set(false);
    }
}

/// Returns the window content with `count` orders and `count` messages.
pub fn view(count: usize) -> impl IntoView {
    let state = State::new();
    let page = Memo::new(move |_| dash::page(state.nav.get()));
    // A page switch drops the views of the old page and builds the new one.
    let content = dyn_container(
        move || page.get(),
        move |page| match page {
            Page::Overview => overview(count, state).into_any(),
            Page::Messages => messages::view(count).into_any(),
            Page::Empty => empty().into_any(),
        },
    )
    .style(|s| s.width_full())
    .scroll()
    .style(|s| {
        s.flex_grow(1.0_f32)
            .min_height(0.0)
            .width_full()
            .overflow_x(Overflow::Hidden)
            .scrollbar_width(px(SCROLLBAR_WIDTH))
    });
    let main = Stack::vertical((topbar::view(state), content))
        .style(|s| s.flex_grow(1.0_f32).min_width(0.0).height_full());
    Stack::horizontal((sidebar::view(state), main)).style(style::window)
}

/// The overview: the metric cards, the chart and filter cards, and the orders.
fn overview(count: usize, state: State) -> impl IntoView {
    Stack::vertical((
        kpis::view(state.live),
        Stack::horizontal((chart::view(state), filters::view(state)))
            .style(|s| s.flex_shrink(0.0_f32).width_full().col_gap(px(dash::GAP))),
        orders::view(count, state),
    ))
    .style(|s| body(s).row_gap(px(dash::GAP)))
}

/// The empty page: its label, centered across the content.
fn empty() -> impl IntoView {
    let label = text(dash::EMPTY_LABEL, dash::FONT_SIZE, Weight::Normal, dash::MUTED)
        .style(|s| s.margin_top(px(dash::EMPTY_TOP - dash::CONTENT_PADDING)));
    Stack::vertical((label,)).style(|s| body(s).items_center())
}

/// The body of a page: the full width of the content with its padding.
fn body(s: floem::style::Style) -> floem::style::Style {
    s.width_full()
        .padding(px(dash::CONTENT_PADDING))
        .padding_right(px(dash::CONTENT_PADDING - SCROLLBAR_WIDTH))
}
