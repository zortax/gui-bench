//! The filter card: egui's checkboxes for the statuses, the compare switch, and the Reset and
//! Apply buttons, top to bottom.

use eframe::egui::{Align, CornerRadius, Layout, Margin, Stroke, Ui, vec2};

use super::spec::{self, Status};
use super::style::{Weight, area, color, fixed_card, primary_button, secondary_button, text};
use super::switch;

/// The top of the filter card in the window.
const CARD_TOP: f32 = spec::TABLE_TOP - spec::GAP - spec::ROW2_HEIGHT;
/// The height of the compare row.
const COMPARE_HEIGHT: f32 = 28.0;
/// The space between the Reset and the Apply button.
const BUTTON_GAP: f32 = spec::APPLY_X - spec::RESET_X - spec::BUTTON_WIDTH;

/// Shows the filter card: `shown` holds the status checkboxes, `compare` the switch. Returns true
/// when Reset was clicked.
pub fn show(ui: &mut Ui, shown: &mut [bool; 4], compare: &mut bool) -> bool {
    let padding = spec::CARD_PADDING;
    let width = spec::FILTERS_WIDTH - 2.0 * padding;
    let mut reset = false;
    let size = vec2(spec::FILTERS_WIDTH, spec::ROW2_HEIGHT);
    fixed_card(ui, size, Margin::same(padding as i8), |ui| {
        let title_height = spec::CHECK_TOP - CARD_TOP - padding;
        area(
            ui,
            vec2(width, title_height),
            Layout::top_down(Align::Min),
            |ui| {
                ui.label(text(
                    spec::FILTERS_TITLE,
                    spec::TITLE_FONT,
                    Weight::Semibold,
                    spec::TEXT,
                ));
            },
        );
        for (status, checked) in Status::ALL.iter().zip(shown.iter_mut()) {
            checkbox(ui, checked, status.label());
        }
        let checks_end = spec::CHECK_TOP + Status::ALL.len() as f32 * spec::CHECK_STRIDE;
        ui.add_space(spec::COMPARE_Y - checks_end);
        let size = vec2(width, COMPARE_HEIGHT);
        area(ui, size, Layout::left_to_right(Align::Center), |ui| {
            ui.label(text(
                spec::COMPARE_LABEL,
                spec::FONT_SIZE,
                Weight::Regular,
                spec::TEXT,
            ));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                switch::toggle(ui, compare);
            });
        });
        ui.add_space(spec::BUTTONS_Y - (spec::COMPARE_Y + COMPARE_HEIGHT));
        let size = vec2(width, spec::CONTROL_HEIGHT);
        area(ui, size, Layout::right_to_left(Align::Center), |ui| {
            let button = vec2(spec::BUTTON_WIDTH, spec::CONTROL_HEIGHT);
            primary_button(ui, spec::APPLY_LABEL, button);
            ui.add_space(BUTTON_GAP);
            reset = secondary_button(ui, spec::RESET_LABEL, button).clicked();
        });
    });
    reset
}

/// One status checkbox, a row of `CHECK_STRIDE`: egui's checkbox with an 18 px box, styled
/// through the spacing and visuals of its scope. egui's checkbox keeps its fill when checked and
/// draws the check mark in the foreground stroke.
fn checkbox(ui: &mut Ui, checked: &mut bool, label: &str) {
    ui.scope(|ui| {
        let spacing = ui.spacing_mut();
        spacing.interact_size.y = spec::CHECK_STRIDE;
        spacing.icon_width = spec::CHECK_SIZE;
        spacing.icon_width_inner = 10.0;
        spacing.icon_spacing = 10.0;
        let visuals = ui.visuals_mut();
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.bg_fill = color(spec::CARD_BG);
            widget.bg_stroke = Stroke::new(1.0, color(spec::INPUT_BORDER));
            widget.fg_stroke = Stroke::new(2.0, color(spec::PRIMARY));
            widget.corner_radius = CornerRadius::same(4);
        }
        ui.checkbox(
            checked,
            text(label, spec::FONT_SIZE, Weight::Regular, spec::TEXT),
        );
    });
}
