//! The chart card: the title, a segmented control of styled buttons in a track (like the filter
//! buttons of iced's `todos` example), and the bar plot: grid lines and bars as plain
//! containers, layered with a `stack`.

use iced::alignment::Vertical;
use iced::widget::{Space, button, column, container, row, space, stack, text};
use iced::{Background, Border, Element, Length, Shadow, border, padding};

use super::style::{self, SEMIBOLD};
use super::{Message, spec};
use crate::page::color;

/// The plot's height, the top of the header row (the segmented control) inside the card, the
/// header's height, the space down to the plot, and the top of the title inside the header.
const PLOT_HEIGHT: f32 = spec::PLOT_BOTTOM - spec::PLOT_TOP;
const HEADER_TOP: f32 = 16.0;
const HEADER_HEIGHT: f32 = spec::SEGMENT_HEIGHT + 4.0;
const PLOT_GAP: f32 = 24.0;
const TITLE_TOP: f32 = spec::CARD_PADDING - HEADER_TOP;
/// The bar of the shown series fills a slot from x 4 for 17 px; with compare, the previous period
/// takes x 3 and the current x 13, both 8 px wide.
const BAR_X: f32 = 4.0;
const BAR_WIDTH: f32 = 17.0;
const PAIR_X: f32 = 3.0;
const PAIR_GAP: f32 = 2.0;
const PAIR_WIDTH: f32 = 8.0;
const BAR_RADIUS: f32 = 4.0;

/// Returns the chart card for series `series`, with the previous period when `compare` is on.
pub fn view<'a>(series: usize, compare: bool) -> Element<'a, Message> {
    let header = row![
        container(
            text(spec::CHART_TITLE)
                .size(spec::TITLE_FONT)
                .font(SEMIBOLD)
                .color(color(spec::TEXT))
        )
        .padding(padding::top(TITLE_TOP))
        .height(Length::Fill),
        space::horizontal(),
        segments(series),
    ]
    .height(HEADER_HEIGHT);
    container(column![header, plot(series, compare)].spacing(PLOT_GAP))
        .padding(padding::all(spec::CARD_PADDING).top(HEADER_TOP))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::card)
        .into()
}

/// Returns the segmented control: a track with one button per segment.
fn segments<'a>(selected: usize) -> Element<'a, Message> {
    let buttons = spec::SEGMENTS.iter().enumerate().map(|(i, label)| {
        let active = i == selected;
        button(
            text(*label)
                .size(13)
                .font(SEMIBOLD)
                .center()
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .padding(0)
        .width(spec::SEGMENT_WIDTH)
        .height(spec::SEGMENT_HEIGHT)
        .on_press(Message::Segment(i))
        .style(move |_, _| button::Style {
            background: active.then(|| Background::Color(color(spec::CARD_BG))),
            text_color: color(if active { spec::TEXT } else { spec::MUTED }),
            border: Border::default().rounded(6),
            shadow: if active {
                style::shadow(spec::SHADOW_SMALL)
            } else {
                Shadow::default()
            },
            snap: true,
        })
        .into()
    });
    container(row(buttons))
        .padding(2)
        .style(|_| style::filled(spec::SEGMENT_TRACK, 8))
        .into()
}

/// Returns the plot: the grid lines behind the bars of the series.
fn plot<'a>(series: usize, compare: bool) -> Element<'a, Message> {
    let mut grid = column![];
    let mut y = spec::PLOT_TOP;
    for line in spec::GRID_LINES {
        grid = grid.push(space().height(line - y)).push(
            container(space())
                .width(Length::Fill)
                .height(1)
                .style(|_| style::filled(spec::GRID, 0)),
        );
        y = line + 1.0;
    }
    let slots = (0..spec::CHART_BARS).map(|i| {
        let current = bar(spec::bar_height(series, i), spec::BAR);
        let slot = if compare {
            row![
                space().width(PAIR_X),
                bar(spec::previous_height(series, i), spec::BAR_PREVIOUS).width(PAIR_WIDTH),
                space().width(PAIR_GAP),
                current.width(PAIR_WIDTH),
            ]
        } else {
            row![space().width(BAR_X), current.width(BAR_WIDTH)]
        };
        slot.align_y(Vertical::Bottom)
            .width(spec::CHART_SLOT)
            .height(Length::Fill)
            .into()
    });
    stack![grid, row(slots)]
        .width(Length::Fill)
        .height(PLOT_HEIGHT)
        .into()
}

/// Returns a bar of `height` with rounded top corners.
fn bar<'a>(height: f32, fill: u32) -> container::Container<'a, Message> {
    container(Space::new())
        .height(height)
        .style(move |_| style::filled(fill, border::top(BAR_RADIUS)))
}
