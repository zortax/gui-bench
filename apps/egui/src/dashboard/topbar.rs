//! The top bar: a top panel whose separator line is its bottom border, with the page title on
//! the left and, from the right end, the avatar, the export button and the search input.

use eframe::egui::{
    Align, Align2, CornerRadius, Frame, Layout, Margin, Panel, Stroke, TextEdit, Ui, Vec2, vec2,
};

use super::spec;
use super::style::{Weight, area, avatar, color, primary_button, text};

/// The space between the top of the bar and its controls.
const CONTROL_TOP: f32 = (spec::TOPBAR_HEIGHT - spec::CONTROL_HEIGHT) / 2.0;
/// The space between the avatar, the export button and the right end.
const SPACING: f32 = 24.0;

/// Shows the top bar with the search input over `search`.
pub fn show(ui: &mut Ui, search: &mut String) {
    // The bar's height includes its 1 px border, the separator line of the panel.
    Panel::top("topbar")
        .exact_size(spec::TOPBAR_HEIGHT)
        .frame(Frame::new().fill(color(spec::TOPBAR_BG)))
        .show(ui, |ui| {
            ui.add_space(CONTROL_TOP);
            let size = vec2(ui.available_width(), spec::CONTROL_HEIGHT);
            area(ui, size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(spec::TOPBAR_TITLE_X - spec::SIDEBAR_WIDTH);
                ui.label(text(
                    spec::PAGE_TITLE,
                    spec::TITLE_FONT + 4.0,
                    Weight::Semibold,
                    spec::TEXT,
                ));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(SPACING);
                    let initials = text(spec::USER_INITIALS, 13.0, Weight::Semibold, 0xffffff);
                    avatar(ui, spec::CONTROL_HEIGHT, spec::USER_COLOR, initials);
                    ui.add_space(SPACING);
                    let export = vec2(spec::EXPORT_WIDTH, spec::CONTROL_HEIGHT);
                    primary_button(ui, spec::EXPORT_LABEL, export);
                    ui.add_space(spec::EXPORT_X - (spec::SEARCH_X + spec::SEARCH_WIDTH));
                    search_input(ui, search);
                });
            });
        });
}

/// The search input: egui's single-line text edit in the spec's border, fill and radius, set
/// through the visuals of its scope.
fn search_input(ui: &mut Ui, search: &mut String) {
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        visuals.text_edit_bg_color = Some(color(spec::INPUT_BG));
        visuals.weak_text_color = Some(color(spec::MUTED));
        visuals.selection.stroke = Stroke::new(1.0, color(spec::PRIMARY));
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.bg_stroke = Stroke::new(1.0, color(spec::INPUT_BORDER));
            widget.corner_radius = CornerRadius::same(8);
            widget.fg_stroke.color = color(spec::TEXT);
        }
        ui.add(
            TextEdit::singleline(search)
                .hint_text(text(
                    spec::SEARCH_PLACEHOLDER,
                    spec::FONT_SIZE,
                    Weight::Regular,
                    spec::MUTED,
                ))
                .margin(Margin::symmetric(12, 0))
                .align(Align2::LEFT_CENTER)
                .desired_width(spec::SEARCH_WIDTH)
                .min_size(Vec2::new(spec::SEARCH_WIDTH, spec::CONTROL_HEIGHT)),
        );
    });
}
