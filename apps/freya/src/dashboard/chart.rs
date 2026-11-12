//! The chart card: the title, freya's `SegmentedButton` for the series, and the bar chart as
//! plain rects over four grid lines.

use freya::prelude::*;

use super::{card, spec, text};
use crate::page::color;

/// The top corners of a bar.
const BAR_RADIUS: CornerRadius = CornerRadius::new(4.0, 4.0, 0.0, 0.0);

/// The chart card. It reads the series and the compare switch, so a change renders the card.
#[derive(PartialEq)]
pub struct Chart {
    pub series: State<usize>,
    pub compare: State<bool>,
}

impl Component for Chart {
    fn render(&self) -> impl IntoElement {
        let series = *self.series.read();
        let compare = *self.compare.read();

        // A narrow window clips the plot to the card.
        card()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .overflow(Overflow::Clip)
            .padding(Gaps::new(
                spec::CARD_PADDING - 4.0,
                spec::CARD_PADDING,
                spec::CARD_PADDING,
                spec::CARD_PADDING,
            ))
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    // The track of the segments: 2 px around them.
                    .height(Size::px(spec::SEGMENT_HEIGHT + 4.0))
                    .main_align(Alignment::SpaceBetween)
                    .child(rect().padding(Gaps::new(4.0, 0.0, 0.0, 0.0)).child(text(
                        spec::CHART_TITLE,
                        spec::TITLE_FONT,
                        FontWeight::SEMI_BOLD,
                        spec::TEXT,
                    )))
                    .child(segments(self.series)),
            )
            .child(plot(series, compare))
    }
}

/// Returns the segmented control on its track: a click on a segment selects its series.
fn segments(mut selected: State<usize>) -> Rect {
    let current = *selected.peek();
    let theme = ButtonSegmentThemePartial::new()
        .background(Color::TRANSPARENT)
        .hover_background(color(spec::RULE))
        .disabled_background(color(spec::SEGMENT_TRACK))
        .selected_background(color(spec::CARD_BG))
        .focus_background(color(spec::RULE))
        .padding(Gaps::new_all(0.0))
        .selected_padding(Gaps::new_all(0.0))
        .width(Size::px(spec::SEGMENT_WIDTH))
        .height(Size::px(spec::SEGMENT_HEIGHT))
        .color(color(spec::MUTED));
    let buttons = spec::SEGMENTS.iter().enumerate().map(|(i, name)| {
        let fill = if i == current {
            spec::TEXT
        } else {
            spec::MUTED
        };
        ButtonSegment::new()
            .key(i)
            .theme(theme.clone())
            .selected(i == current)
            .on_press(move |_| selected.set(i))
            .child(text(*name, 13.0, FontWeight::SEMI_BOLD, fill))
    });

    rect()
        .padding(2.0)
        .corner_radius(8.0)
        .background(color(spec::SEGMENT_TRACK))
        .child(
            SegmentedButton::new()
                .theme(
                    SegmentedButtonThemePartial::new()
                        .background(Color::TRANSPARENT)
                        .border_fill(Color::TRANSPARENT)
                        .corner_radius(6.0),
                )
                .children(buttons),
        )
}

/// Returns the plot: the grid lines and one slot per bar, with the bars on the baseline.
fn plot(series: usize, compare: bool) -> Rect {
    let grid = spec::GRID_LINES.iter().map(|&y| {
        rect()
            .position(Position::new_absolute().top(y - spec::PLOT_TOP).left(0.0))
            .width(Size::fill())
            .height(Size::px(1.0))
            .background(color(spec::GRID))
    });
    let slots = (0..spec::CHART_BARS).map(move |i| {
        let current = bar(spec::bar_height(series, i), spec::BAR);
        let slot = rect()
            .horizontal()
            .width(Size::px(spec::CHART_SLOT))
            .height(Size::fill())
            .cross_align(Alignment::End);
        if compare {
            slot.padding(Gaps::new(0.0, 0.0, 0.0, 3.0))
                .spacing(2.0)
                .child(
                    bar(spec::previous_height(series, i), spec::BAR_PREVIOUS).width(Size::px(8.0)),
                )
                .child(current.width(Size::px(8.0)))
        } else {
            slot.padding(Gaps::new(0.0, 0.0, 0.0, 4.0))
                .child(current.width(Size::px(17.0)))
        }
    });

    rect()
        .width(Size::fill())
        .height(Size::px(spec::PLOT_BOTTOM - spec::PLOT_TOP))
        .margin(Gaps::new(24.0, 0.0, 0.0, 0.0))
        .children(grid)
        .child(
            rect()
                .horizontal()
                .width(Size::fill())
                .height(Size::fill())
                .children(slots),
        )
}

/// Returns a bar of `height` in `fill`, rounded at the top.
fn bar(height: f32, fill: u32) -> Rect {
    rect()
        .height(Size::px(height))
        .corner_radius(BAR_RADIUS)
        .background(color(fill))
}
