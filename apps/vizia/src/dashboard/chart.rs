//! The chart card: the title, the segmented control (a `ButtonGroup` of `ToggleButton`s, as the
//! default theme styles them) and the plot. The plot holds the grid lines and one slot per bar;
//! the bar heights map the series signal, and the `compare` class from the compare signal shows
//! the previous-period bars and narrows the current ones.

use vizia::prelude::*;

use super::{Dashboard, DashboardEvent};
use crate::dashboard_spec::{
    CHART_BARS, CHART_TITLE, GRID_LINES, PLOT_TOP, SEGMENTS, bar_height, previous_height,
};

/// Builds the chart card.
pub fn build(cx: &mut Context, state: Dashboard) {
    VStack::new(cx, move |cx| {
        HStack::new(cx, move |cx| {
            Label::new(cx, CHART_TITLE).class("card-title");
            ButtonGroup::new(cx, move |cx| {
                for (k, name) in SEGMENTS.into_iter().enumerate() {
                    ToggleButton::new(cx, state.series.map(move |s| *s == k), move |cx| {
                        Label::new(cx, name)
                    })
                    .on_toggle(move |cx| cx.emit(DashboardEvent::Series(k)));
                }
            })
            .class("segments");
        })
        .class("chart-head");

        VStack::new(cx, move |cx| {
            for y in GRID_LINES {
                Element::new(cx)
                    .class("grid-line")
                    .top(Pixels(y - PLOT_TOP));
            }
            HStack::new(cx, move |cx| {
                for i in 0..CHART_BARS {
                    slot(cx, state, i);
                }
            })
            .class("bars");
        })
        .class("plot")
        .toggle_class("compare", state.compare);
    })
    .class("card")
    .class("chart");
}

/// Builds slot `i` of the plot: the previous-period bar (shown with compare) and the bar of the
/// selected series.
fn slot(cx: &mut Context, state: Dashboard, i: usize) {
    HStack::new(cx, move |cx| {
        Element::new(cx)
            .class("previous")
            .height(state.series.map(move |s| Pixels(previous_height(*s, i))));
        Element::new(cx)
            .class("bar")
            .height(state.series.map(move |s| Pixels(bar_height(*s, i))));
    })
    .class("slot");
}
