//! The chart card: the title, gpui-component's toggle group as the segmented control, and a plot
//! of grid lines and plain rounded bars, with the previous period beside them when compare is on.

use gpui_kit::component::button::{Toggle, ToggleGroup};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::spec;
use super::style::{card, shadow};

/// The height of the plot, from its top to the baseline.
const PLOT_HEIGHT: f32 = spec::PLOT_BOTTOM - spec::PLOT_TOP;
/// The plot top below the card top.
const PLOT_OFFSET: f32 = spec::PLOT_TOP - (spec::TABLE_TOP - spec::GAP - spec::ROW2_HEIGHT);

/// Returns the chart card for `series`, with the previous period when `compare`. A click on a
/// segment calls `on_select` with the checked state of every segment.
pub fn chart(
    series: usize,
    compare: bool,
    on_select: impl Fn(&Vec<bool>, &mut Window, &mut App) + 'static,
) -> Div {
    card()
        .relative()
        .flex_1()
        .min_w_0()
        .h_full()
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .left(px(spec::CARD_PADDING))
                .top(px(spec::CARD_PADDING))
                .text_size(px(spec::TITLE_FONT))
                .font_weight(FontWeight::SEMIBOLD)
                .child(spec::CHART_TITLE),
        )
        .child(
            div()
                .absolute()
                .top(px(16.0))
                .right(px(spec::CARD_PADDING))
                .child(segments(series, on_select)),
        )
        .child(plot(series, compare))
}

/// Returns the segmented control: a single-choice toggle group on the segment track, the
/// selected toggle white and raised.
fn segments(
    series: usize,
    on_select: impl Fn(&Vec<bool>, &mut Window, &mut App) + 'static,
) -> ToggleGroup {
    let toggles = spec::SEGMENTS.iter().enumerate().map(|(i, label)| {
        let selected = i == series;
        Toggle::new(("segment", i))
            .label(*label)
            .checked(selected)
            .w(px(spec::SEGMENT_WIDTH))
            .h(px(spec::SEGMENT_HEIGHT))
            .rounded(px(6.0))
            .text_size(px(13.0))
            .font_weight(FontWeight::SEMIBOLD)
            .when(selected, |toggle| {
                toggle
                    .bg(rgb(spec::CARD_BG))
                    .text_color(rgb(spec::TEXT))
                    .shadow(shadow(spec::SHADOW_SMALL))
            })
            .when(!selected, |toggle| toggle.text_color(rgb(spec::MUTED)))
    });
    ToggleGroup::new("segments")
        .segmented()
        .single()
        .children(toggles)
        .p(px(2.0))
        .rounded(px(8.0))
        .bg(rgb(spec::SEGMENT_TRACK))
        .on_click(on_select)
}

/// Returns the plot: the grid lines behind the bars of the series.
fn plot(series: usize, compare: bool) -> Div {
    let grid = spec::GRID_LINES.iter().map(|y| {
        div()
            .absolute()
            .left_0()
            .right_0()
            .top(px(y - spec::PLOT_TOP))
            .h(px(1.0))
            .bg(rgb(spec::GRID))
    });
    let bars = (0..spec::CHART_BARS).flat_map(move |i| {
        let slot = i as f32 * spec::CHART_SLOT;
        let current = spec::bar_height(series, i);
        if compare {
            vec![
                bar(
                    slot + 3.0,
                    8.0,
                    spec::previous_height(series, i),
                    spec::BAR_PREVIOUS,
                ),
                bar(slot + 13.0, 8.0, current, spec::BAR),
            ]
        } else {
            vec![bar(slot + 4.0, 17.0, current, spec::BAR)]
        }
    });
    div()
        .absolute()
        .left(px(spec::CARD_PADDING))
        .right(px(spec::CARD_PADDING))
        .top(px(PLOT_OFFSET))
        .h(px(PLOT_HEIGHT))
        .children(grid)
        .children(bars)
}

/// Returns a bar standing on the baseline at `x` in the plot.
fn bar(x: f32, width: f32, height: f32, color: u32) -> Div {
    div()
        .absolute()
        .left(px(x))
        .bottom_0()
        .w(px(width))
        .h(px(height))
        .rounded_t(px(4.0))
        .bg(rgb(color))
}
