//! The dashboard scenario: a sidebar, a top bar and a scroll container with the metric cards, the
//! chart and filter cards and the orders table of `apps/shared/dashboard.rs`. The state lives in
//! floem signals that the views read in their closures; the scroll container scrolls as floem's
//! does by default, like the rows scenario.

mod chart;
mod filters;
mod kpis;
mod orders;
mod sidebar;
mod style;
mod topbar;

use floem::prelude::*;
use floem::taffy::Overflow;

use crate::dash;
use style::{color, px};

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
}

impl State {
    fn new() -> Self {
        Self {
            nav: RwSignal::new(0),
            search: RwSignal::new(String::new()),
            series: RwSignal::new(0),
            compare: RwSignal::new(false),
            shown: std::array::from_fn(|_| RwSignal::new(true)),
        }
    }

    /// Returns the status filter, and makes the caller track it.
    fn shown(self) -> [bool; 4] {
        self.shown.map(|shown| shown.get())
    }

    /// The Reset button: every status checked, compare off, the first series, no search.
    fn reset(self) {
        for shown in self.shown {
            shown.set(true);
        }
        self.compare.set(false);
        self.series.set(0);
        self.search.set(String::new());
    }
}

/// Returns the window content with `count` orders.
pub fn view(count: usize) -> impl IntoView {
    let state = State::new();
    let content = Stack::vertical((
        kpis::view(),
        Stack::horizontal((chart::view(state), filters::view(state)))
            .style(|s| s.flex_shrink(0.0_f32).width_full().col_gap(px(dash::GAP))),
        orders::view(count, state),
    ))
    .style(|s| {
        s.width_full()
            .padding(px(dash::CONTENT_PADDING))
            .padding_right(px(dash::CONTENT_PADDING - SCROLLBAR_WIDTH))
            .row_gap(px(dash::GAP))
    })
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
    Stack::horizontal((sidebar::view(state), main)).style(|s| {
        s.size_full()
            .background(color(dash::PAGE_BG))
            .color(color(dash::TEXT))
            .font_family(crate::spec::FONT_FAMILY.to_owned())
            .font_size(px(dash::FONT_SIZE))
            .selectable(false)
    })
}
