//! The switch of the compare row and of the live updates. egui has no switch widget; its docs point to the toggle switch of the egui
//! demo (`egui_demo_lib/src/demo/toggle_switch.rs`), and this is that widget in the spec's size
//! and colors: allocate the rect, toggle on click, paint the track and the knob.

use eframe::egui::{
    self, Rect, Response, Role, Sense, StrokeKind, Ui, Vec2, WidgetInfo, pos2, vec2,
};

use super::spec;
use super::style::{color, shadow};

/// The space between the knob and the ends of the track.
const KNOB_INSET: f32 = 2.0;

/// Shows the switch over `on`; a click toggles it.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> Response {
    let size = vec2(spec::SWITCH_WIDTH, spec::SWITCH_HEIGHT);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::selected(Role::Switch, ui.is_enabled(), *on, ""));
    if ui.is_rect_visible(rect) {
        // The style turns animations off, so this is the new position at once.
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let track = if *on { spec::PRIMARY } else { spec::SWITCH_OFF };
        let radius = rect.height() / 2.0;
        let painter = ui.painter();
        painter.rect(
            rect,
            radius,
            color(track),
            egui::Stroke::NONE,
            StrokeKind::Inside,
        );
        let knob_radius = radius - KNOB_INSET;
        let x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = pos2(x, rect.center().y);
        let knob = Rect::from_center_size(center, Vec2::splat(2.0 * knob_radius));
        painter.add(shadow(spec::SHADOW_SMALL).as_shape(knob, knob_radius));
        painter.circle_filled(center, knob_radius, color(spec::PRIMARY_TEXT));
    }
    response
}
