//! The chart card: the title, the segmented control of the series (egui's selectable buttons in
//! a track frame) and the plot, plain bars laid out left to right on the baseline over four grid
//! lines.

use eframe::egui::{
    Align, Button, Color32, CornerRadius, Frame, Layout, Margin, Rect, Sense, Stroke, Ui,
    UiBuilder, pos2, vec2,
};

use super::spec;
use super::style::{Weight, area, block, color, fixed_card, shadow, text};

/// The space between the top of the card and the segmented control.
const HEADER_TOP: f32 = 16.0;
/// The height of the header row: the segmented control with its track padding.
const HEADER_HEIGHT: f32 = spec::SEGMENT_HEIGHT + 2.0 * TRACK_PADDING;
const TRACK_PADDING: f32 = 2.0;
/// The top of the title in the card.
const TITLE_Y: f32 = 20.0;
/// The top of the chart card in the window.
const CARD_TOP: f32 = spec::TABLE_TOP - spec::GAP - spec::ROW2_HEIGHT;

/// Shows the chart card in the available width with the series of `segment`, which a click on a
/// segment changes, and the previous period beside it when `compare` is on.
pub fn show(ui: &mut Ui, segment: &mut usize, compare: bool) {
    let padding = spec::CARD_PADDING;
    let inner_width = ui.available_width() - 2.0 * padding;
    let margin = Margin {
        left: padding as i8,
        right: padding as i8,
        top: HEADER_TOP as i8,
        bottom: padding as i8,
    };
    let size = vec2(ui.available_width(), spec::ROW2_HEIGHT);
    fixed_card(ui, size, margin, |ui| {
        let size = vec2(inner_width, HEADER_HEIGHT);
        area(ui, size, Layout::left_to_right(Align::Min), |ui| {
            ui.vertical(|ui| {
                ui.add_space(TITLE_Y - HEADER_TOP);
                ui.label(text(
                    spec::CHART_TITLE,
                    spec::TITLE_FONT,
                    Weight::Semibold,
                    spec::TEXT,
                ));
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                segments(ui, segment);
            });
        });
        ui.add_space(spec::PLOT_TOP - CARD_TOP - HEADER_TOP - HEADER_HEIGHT);
        plot(ui, inner_width, *segment, compare);
    });
}

/// The segmented control: a track frame around one selectable button per series. The selected
/// one is white with the small shadow of a frame around it.
fn segments(ui: &mut Ui, segment: &mut usize) {
    let track = Frame::new()
        .fill(color(spec::SEGMENT_TRACK))
        .corner_radius(8)
        .inner_margin(Margin::same(TRACK_PADDING as i8));
    track.show(ui, |ui| {
        let visuals = ui.visuals_mut();
        visuals.selection.bg_fill = color(spec::CARD_BG);
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.weak_bg_fill = Color32::TRANSPARENT;
            widget.bg_stroke = Stroke::NONE;
        }
        let count = spec::SEGMENTS.len() as f32;
        let size = vec2(count * spec::SEGMENT_WIDTH, spec::SEGMENT_HEIGHT);
        area(ui, size, Layout::left_to_right(Align::Center), |ui| {
            for (i, name) in spec::SEGMENTS.iter().enumerate() {
                let selected = *segment == i;
                let label_color = if selected { spec::TEXT } else { spec::MUTED };
                let button =
                    Button::selectable(selected, text(*name, 13.0, Weight::Semibold, label_color))
                        .corner_radius(6)
                        .min_size(vec2(spec::SEGMENT_WIDTH, spec::SEGMENT_HEIGHT));
                let response = if selected {
                    Frame::new()
                        .corner_radius(6)
                        .shadow(shadow(spec::SHADOW_SMALL))
                        .show(ui, |ui| ui.add(button))
                        .inner
                } else {
                    ui.add(button)
                };
                if response.clicked() {
                    *segment = i;
                }
            }
        });
    });
}

/// The plot, `width` wide: the grid lines, then the bars of `series` from the left, each in its
/// slot; with `compare`, a narrow bar of the previous period before each narrow current one.
fn plot(ui: &mut Ui, width: f32, series: usize, compare: bool) {
    let height = spec::PLOT_BOTTOM - spec::PLOT_TOP;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    for y in spec::GRID_LINES {
        let top = rect.top() + y - spec::PLOT_TOP;
        let line = Rect::from_min_size(pos2(rect.left(), top), vec2(width, 1.0));
        ui.painter().rect_filled(line, 0, color(spec::GRID));
    }
    let top = CornerRadius {
        nw: 4,
        ne: 4,
        sw: 0,
        se: 0,
    };
    let bars = UiBuilder::new()
        .max_rect(rect)
        .layout(Layout::left_to_right(Align::Max));
    ui.scope_builder(bars, |ui| {
        ui.shrink_clip_rect(rect);
        for i in 0..spec::CHART_BARS {
            let current = spec::bar_height(series, i);
            if compare {
                let previous = spec::previous_height(series, i);
                ui.add_space(3.0);
                block(ui, vec2(8.0, previous), spec::BAR_PREVIOUS, top);
                ui.add_space(2.0);
                block(ui, vec2(8.0, current), spec::BAR, top);
                ui.add_space(4.0);
            } else {
                ui.add_space(4.0);
                block(ui, vec2(17.0, current), spec::BAR, top);
                ui.add_space(4.0);
            }
        }
    });
}
