//! The chart card: the title, a segmented control of floem buttons that picks the series, and the
//! plot of plain rounded bars over four grid lines. The bar styles read the series and compare
//! signals, so a change restyles the bars in place.

use floem::prelude::*;

use super::State;
use super::style::{self, Weight, color, px, text};
use crate::dash;

/// The space between the header and the plot.
const HEADER_GAP: f32 = 24.0;

/// Returns the chart card.
pub fn view(state: State) -> impl IntoView {
    let title = text(dash::CHART_TITLE, dash::TITLE_FONT, Weight::Semibold, dash::TEXT)
        .style(|s| s.margin_top(4.0));
    let header = Stack::horizontal((title, segments(state))).style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .justify_between()
            .items_start()
    });
    Stack::vertical((header, plot(state))).style(|s| {
        style::card(s)
            .flex_grow(1.0_f32)
            .flex_shrink(1.0_f32)
            .flex_basis(px(dash::CHART_WIDTH))
            .min_width(0.0)
            .height(px(dash::ROW2_HEIGHT))
            .padding(px(dash::CARD_PADDING))
            .padding_top(16.0)
            .row_gap(px(HEADER_GAP))
    })
}

/// The segmented control: one floem button per series in a gray track; the selected one is
/// white with the small shadow.
fn segments(state: State) -> impl IntoView {
    Stack::horizontal_from_iter(dash::SEGMENTS.iter().enumerate().map(|(i, label)| {
        let selected = move || state.series.get() == i;
        let label = Label::new(label).style(move |s| {
            let hex = if selected() { dash::TEXT } else { dash::MUTED };
            s.font_size(13.0)
                .font_weight(floem::text::FontWeight::SEMI_BOLD)
                .color(color(hex))
        });
        Button::new(label)
            .action(move || state.series.set(i))
            .style(move |s| {
                let background = if selected() {
                    color(dash::CARD_BG)
                } else {
                    Color::TRANSPARENT
                };
                let s = s
                    .size(px(dash::SEGMENT_WIDTH), px(dash::SEGMENT_HEIGHT))
                    .padding(0.0)
                    .border(0.0)
                    .border_radius(6.0)
                    .background(background)
                    .hover(|s| s.background(background))
                    .active(|s| s.background(background));
                s.apply_if(selected(), |s| style::shadow(s, dash::SHADOW_SMALL))
            })
    }))
    .style(|s| {
        s.flex_shrink(0.0_f32)
            .padding(2.0)
            .border_radius(8.0)
            .background(color(dash::SEGMENT_TRACK))
    })
}

/// The plot: the grid lines, and in front of them one slot per bar with the bars at its bottom.
fn plot(state: State) -> impl IntoView {
    let grid = Stack::from_iter(dash::GRID_LINES.iter().map(|&y| {
        Empty::new().style(move |s| {
            s.absolute()
                .inset_left(0.0)
                .inset_right(0.0)
                .inset_top(px(y - dash::PLOT_TOP))
                .height(1.0)
                .background(color(dash::GRID))
        })
    }))
    .style(|s| s.absolute().inset(0.0));
    let bars = Stack::horizontal_from_iter((0..dash::CHART_BARS).map(|i| slot(i, state)))
        .style(|s| s.absolute().inset(0.0));
    Stack::new((grid, bars)).style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(px(dash::PLOT_BOTTOM - dash::PLOT_TOP))
            .overflow_x(floem::taffy::Overflow::Hidden)
    })
}

/// Slot `i`: the bar of the shown series, and with compare on the previous period beside it.
fn slot(i: usize, state: State) -> impl IntoView {
    let previous = Empty::new().style(move |s| {
        let series = state.series.get();
        bar(s, dash::previous_height(series, i), dash::BAR_PREVIOUS)
            .width(8.0)
            .margin_right(2.0)
            .apply_if(!state.compare.get(), |s| s.hide())
    });
    let current = Empty::new().style(move |s| {
        let width = if state.compare.get() { 8.0 } else { 17.0 };
        bar(s, dash::bar_height(state.series.get(), i), dash::BAR).width(width)
    });
    Stack::horizontal((previous, current)).style(move |s| {
        let left = if state.compare.get() { 3.0 } else { 4.0 };
        s.flex_shrink(0.0_f32)
            .width(px(dash::CHART_SLOT))
            .height_full()
            .padding_left(left)
            .items_end()
    })
}

/// A bar of `height` in `hex` with rounded top corners.
fn bar(s: floem::style::Style, height: f32, hex: u32) -> floem::style::Style {
    s.flex_shrink(0.0_f32)
        .height(px(height))
        .border_top_left_radius(4.0)
        .border_top_right_radius(4.0)
        .background(color(hex))
}
