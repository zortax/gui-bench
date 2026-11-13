//! The metric cards: four cards in a row that share the width, each with its label, value, delta
//! pill and a sparkline of plain bars at the right.

use eframe::egui::{Align, Layout, Margin, Ui, Vec2, vec2};

use super::spec::{self, Kpi};
use super::style::{Weight, area, block, fixed_card, pill, text};

/// The top of the label, the value and the delta pill in a card.
const LABEL_Y: f32 = 18.0;
const VALUE_Y: f32 = 40.0;
const DELTA_Y: f32 = 78.0;
const DELTA_HEIGHT: f32 = 22.0;
/// The bottom of the sparkline bars in a card.
const SPARK_BOTTOM: f32 = 92.0;

/// Shows the row of metric cards with their values at live tick `tick`.
pub fn show(ui: &mut Ui, tick: u64) {
    let count = spec::KPI_COUNT as f32;
    let width = (ui.available_width() - (count - 1.0) * spec::GAP) / count;
    ui.horizontal(|ui| {
        for (k, kpi) in spec::KPIS.iter().enumerate() {
            if k > 0 {
                ui.add_space(spec::GAP);
            }
            metric(ui, kpi, &spec::kpi_value(k, tick), width);
        }
    });
}

/// One metric card, `width` wide, that shows `value`.
fn metric(ui: &mut Ui, kpi: &Kpi, value: &str, width: f32) {
    let padding = spec::CARD_PADDING;
    let margin = Margin {
        left: padding as i8,
        right: padding as i8,
        top: LABEL_Y as i8,
        bottom: 0,
    };
    let size = vec2(width, spec::KPI_HEIGHT);
    fixed_card(ui, size, margin, |ui| {
        let size = vec2(width - 2.0 * padding, spec::KPI_HEIGHT - LABEL_Y);
        area(ui, size, Layout::left_to_right(Align::Min), |ui| {
            ui.vertical(|ui| {
                let label = text(kpi.label, 13.0, Weight::Regular, spec::MUTED);
                area(
                    ui,
                    vec2(0.0, VALUE_Y - LABEL_Y),
                    Layout::top_down(Align::Min),
                    |ui| ui.label(label),
                );
                let value = text(value, 24.0, Weight::Bold, spec::TEXT);
                area(
                    ui,
                    vec2(0.0, DELTA_Y - VALUE_Y),
                    Layout::top_down(Align::Min),
                    |ui| ui.label(value),
                );
                let (bg, fg) = if kpi.up {
                    (spec::UP_BG, spec::UP_TEXT)
                } else {
                    (spec::DOWN_BG, spec::DOWN_TEXT)
                };
                pill(
                    ui,
                    text(kpi.delta, 12.0, Weight::Semibold, fg),
                    bg,
                    DELTA_HEIGHT,
                    8.0,
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                sparkline(ui, &kpi.spark);
            });
        });
    });
}

/// The sparkline: bars from the top of the label row down to their common bottom, the newest at
/// the right.
fn sparkline(ui: &mut Ui, heights: &[f32]) {
    let n = heights.len() as f32;
    let width = spec::SPARK_STEP * (n - 1.0) + spec::SPARK_BAR_WIDTH;
    let size = vec2(width, SPARK_BOTTOM - LABEL_Y);
    area(ui, size, Layout::left_to_right(Align::Max), |ui| {
        for (i, &height) in heights.iter().enumerate() {
            if i > 0 {
                ui.add_space(spec::SPARK_STEP - spec::SPARK_BAR_WIDTH);
            }
            let bar = Vec2::new(spec::SPARK_BAR_WIDTH, height);
            block(ui, bar, spec::SPARK, 2);
        }
    });
}
