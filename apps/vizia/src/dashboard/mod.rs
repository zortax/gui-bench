//! The app scenario: the sidebar, the top bar and a scroll view with the page of the active nav
//! item: the overview (the metric cards, the chart and filter cards and the orders card), the
//! messages feed, or an empty page. Stacks lay everything out and `dashboard.css` styles it.
//!
//! The state is a model of signals: the views read the signals they show, and the controls emit
//! `DashboardEvent`s, which the model turns into signal updates. The content scrolls in the stock
//! scroll view.

mod chart;
mod filters;
mod kpis;
pub mod list;
mod live;
mod messages;
mod orders;
mod sidebar;
mod topbar;

use vizia::prelude::*;

use self::live::Live;
use crate::dashboard_spec::{EMPTY_LABEL, Page, page};

/// The state of the dashboard. Every field is a signal, so the state is a copyable handle that
/// the builders share.
#[derive(Clone, Copy)]
pub struct Dashboard {
    /// The active nav item.
    nav: Signal<usize>,
    /// The text of the search input.
    search: Signal<String>,
    /// The series the chart shows (an index into `SEGMENTS`).
    series: Signal<usize>,
    /// Whether the chart shows the previous period beside the current one.
    compare: Signal<bool>,
    /// Whether the orders of `Status::ALL[k]` are shown.
    shown: [Signal<bool>; 4],
    /// Whether the orders are sorted by amount.
    sorted: Signal<bool>,
    /// The live updates.
    live: Live,
}

/// The events of the controls.
pub enum DashboardEvent {
    /// A nav item was clicked.
    Navigate(usize),
    /// The search text was edited.
    Search(String),
    /// A segment of the chart was clicked.
    Series(usize),
    /// The compare switch was clicked.
    ToggleCompare,
    /// The checkbox of a status was clicked.
    ToggleStatus(usize),
    /// The live switch was clicked.
    ToggleLive,
    /// Reset was clicked: every status shown, compare off, the first series, no search, no
    /// sorting.
    Reset,
}

impl Model for Dashboard {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|dashboard_event, _| match dashboard_event {
            DashboardEvent::Navigate(item) => self.nav.set(*item),
            DashboardEvent::Search(text) => self.search.set(text.clone()),
            DashboardEvent::Series(series) => self.series.set(*series),
            DashboardEvent::ToggleCompare => self.compare.update(|on| *on = !*on),
            DashboardEvent::ToggleStatus(k) => self.shown[*k].update(|on| *on = !*on),
            DashboardEvent::ToggleLive => self.live.toggle(cx),
            DashboardEvent::Reset => {
                for shown in self.shown {
                    shown.set(true);
                }
                self.compare.set(false);
                self.series.set(0);
                self.search.set(String::new());
                self.sorted.set(false);
            }
        });
    }
}

/// Builds the dashboard with `count` orders and `count` messages and adds its stylesheet.
pub fn build(cx: &mut Context, count: usize) {
    add_stylesheet(cx);
    let state = Dashboard {
        nav: Signal::new(0),
        search: Signal::new(String::new()),
        series: Signal::new(0),
        compare: Signal::new(false),
        shown: [(); 4].map(|_| Signal::new(true)),
        sorted: Signal::new(false),
        live: Live::new(cx),
    };
    state.build(cx);

    HStack::new(cx, move |cx| {
        sidebar::build(cx, state);
        VStack::new(cx, move |cx| {
            topbar::build(cx, state);
            ScrollView::new(cx, move |cx| content(cx, state, count))
                .show_horizontal_scrollbar(false)
                .class("content");
        })
        .class("main");
    })
    .class("dashboard");
}

/// Builds the page of the active nav item. The binding removes the page and builds the new one
/// when the nav item names another page.
fn content(cx: &mut Context, state: Dashboard, count: usize) {
    let active = state.nav.map(|item| page(*item));
    Binding::new(cx, active, move |cx| match active.get() {
        Page::Overview => {
            VStack::new(cx, move |cx| {
                kpis::build(cx, state.live);
                HStack::new(cx, move |cx| {
                    chart::build(cx, state);
                    filters::build(cx, state);
                })
                .class("row2");
                orders::build(cx, state, count);
            })
            .class("blocks");
        }
        Page::Messages => messages::build(cx, count),
        Page::Empty => {
            VStack::new(cx, |cx| {
                Label::new(cx, EMPTY_LABEL).class("empty-label");
            })
            .class("blocks")
            .class("empty");
        }
    });
}

/// Adds the stylesheet of the dashboard.
fn add_stylesheet(cx: &mut Context) {
    cx.add_stylesheet(include_style!("src/dashboard/dashboard.css"))
        .expect("failed to add the dashboard stylesheet");
}

/// Returns the color of a `0xrrggbb` value of the spec.
fn rgb(value: u32) -> Color {
    Color::rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
