//! The look of the dashboard: the egui style of the window, the font weights, the spec colors and
//! shadows in egui types, and the small styled pieces that several parts share (a fixed block, a
//! pill, a round avatar, the primary and secondary buttons).

use eframe::egui::{
    self, Align, Button, Color32, Context, CornerRadius, FontFamily, Frame, Label, Layout, Margin,
    Rect, Response, RichText, Sense, Shadow, Stroke, TextStyle, TextWrapMode, Theme, Ui, UiBuilder,
    Vec2, WidgetText, vec2,
};
use fontique::FontWeight;

use super::spec;
use crate::font;

/// Returns the egui color of a spec color.
pub const fn color(color: u32) -> Color32 {
    let (r, g, b) = rgb(color);
    Color32::from_rgb(r, g, b)
}

/// Returns the red, green and blue of a `0xrrggbb` color.
const fn rgb(color: u32) -> (u8, u8, u8) {
    ((color >> 16) as u8, (color >> 8) as u8, color as u8)
}

/// Returns the egui shadow of a spec shadow. A CSS blur radius blurs the edge over twice its
/// length, and the blur of an epaint shadow is the whole width of that penumbra.
pub fn shadow(shadow: spec::Shadow) -> Shadow {
    let (r, g, b) = rgb(shadow.color);
    Shadow {
        offset: [shadow.x as i8, shadow.y as i8],
        blur: (2.0 * shadow.blur) as u8,
        spread: 0,
        color: Color32::from_rgba_unmultiplied(r, g, b, (shadow.alpha * 255.0).round() as u8),
    }
}

/// The weight of a text. egui has no weights, so each bolder one is a font family of its own.
#[derive(Clone, Copy)]
pub enum Weight {
    Regular,
    Semibold,
    Bold,
}

const SEMIBOLD: &str = "semibold";
const BOLD: &str = "bold";

/// Returns `text` in `size` px, `weight` and `color`.
pub fn text(text: impl Into<String>, size: f32, weight: Weight, color: u32) -> RichText {
    let family = match weight {
        Weight::Regular => FontFamily::Proportional,
        Weight::Semibold => FontFamily::Name(SEMIBOLD.into()),
        Weight::Bold => FontFamily::Name(BOLD.into()),
    };
    RichText::new(text)
        .size(size)
        .family(family)
        .color(self::color(color))
}

/// Sets the light theme, the system font with its semibold and bold faces, 14 px text in the
/// spec text color that never wraps, no item spacing, no animation, widgets that keep their size on hover, labels
/// that do not select text, and the top bar rule as the separator color. The scroll bar stays
/// egui's floating one, which takes no room from the content.
pub fn setup(ctx: &Context) {
    ctx.set_theme(Theme::Light);
    ctx.set_fonts(font::with_weights(&[
        (SEMIBOLD, FontWeight::SEMI_BOLD),
        (BOLD, FontWeight::BOLD),
    ]));
    ctx.all_styles_mut(|style| {
        style.animation_time = 0.0;
        style.spacing.item_spacing = Vec2::ZERO;
        style.interaction.selectable_labels = false;
        style.wrap_mode = Some(TextWrapMode::Extend);
        let visuals = &mut style.visuals;
        visuals.panel_fill = color(spec::PAGE_BG);
        visuals.widgets.noninteractive.fg_stroke.color = color(spec::TEXT);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, color(spec::RULE));
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.expansion = 0.0;
        }
        if let Some(body) = style.text_styles.get_mut(&TextStyle::Body) {
            body.size = spec::FONT_SIZE;
        }
    });
}

/// Adds a `size` block of `fill` with `radius`: a plain rectangle laid out like any widget.
pub fn block(ui: &mut Ui, size: Vec2, fill: u32, radius: impl Into<CornerRadius>) -> Rect {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, radius, color(fill));
    rect
}

/// Adds a pill: `label` on `bg`, `height` tall and fully rounded, with `padding` on the sides.
pub fn pill(ui: &mut Ui, label: RichText, bg: u32, height: f32, padding: f32) -> Response {
    Frame::new()
        .fill(color(bg))
        .corner_radius(height / 2.0)
        .inner_margin(Margin::symmetric(padding as i8, 0))
        .show(ui, |ui| {
            // The label, laid out first to know its width, centered in the pill's height.
            let galley = WidgetText::from(label).into_galley(
                ui,
                Some(TextWrapMode::Extend),
                f32::INFINITY,
                TextStyle::Body,
            );
            let size = vec2(galley.size().x, height);
            ui.add_sized(size, Label::new(galley));
        })
        .response
}

/// Adds a round avatar of `size` in `fill` with `initials` in its center.
pub fn avatar(ui: &mut Ui, size: f32, fill: u32, initials: RichText) -> Response {
    Frame::new()
        .fill(color(fill))
        .corner_radius(size / 2.0)
        .show(ui, |ui| {
            ui.add_sized(Vec2::splat(size), Label::new(initials))
        })
        .response
}

/// Adds a primary button, `size` large: white semibold text on the primary color, with the small
/// shadow of a frame around it.
pub fn primary_button(ui: &mut Ui, label: &str, size: Vec2) -> Response {
    Frame::new()
        .corner_radius(8)
        .shadow(shadow(spec::SHADOW_SMALL))
        .show(ui, |ui| {
            ui.add(
                Button::new(text(
                    label,
                    spec::FONT_SIZE,
                    Weight::Semibold,
                    spec::PRIMARY_TEXT,
                ))
                .fill(color(spec::PRIMARY))
                .stroke(Stroke::NONE)
                .corner_radius(8)
                .min_size(size),
            )
        })
        .inner
}

/// Adds a secondary button, `size` large: semibold text on white with a gray border.
pub fn secondary_button(ui: &mut Ui, label: &str, size: Vec2) -> Response {
    ui.add(
        Button::new(text(label, spec::FONT_SIZE, Weight::Semibold, spec::TEXT))
            .fill(color(spec::SECONDARY_BG))
            .stroke(Stroke::new(1.0, color(spec::SECONDARY_BORDER)))
            .corner_radius(8)
            .min_size(size),
    )
}

/// The frame of every card: white, rounded, with the card shadow.
pub fn card() -> Frame {
    Frame::new()
        .fill(color(spec::CARD_BG))
        .corner_radius(spec::CARD_RADIUS)
        .shadow(shadow(spec::SHADOW_CARD))
}

/// Shows a card of exactly `size` with `margin`, and `add` top to bottom inside it. Content that
/// does not fit is clipped instead of growing the card, so a narrow window shrinks the cards
/// that share a row: the card takes its rect from the layout and paints its frame there, and the
/// content goes into a child ui over that rect.
pub fn fixed_card<R>(ui: &mut Ui, size: Vec2, margin: Margin, add: impl FnOnce(&mut Ui) -> R) -> R {
    let frame = card().inner_margin(margin);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let content = rect - frame.total_margin();
    if ui.is_rect_visible(rect) {
        ui.painter().add(frame.paint(content));
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(content)
            .layout(Layout::top_down(Align::Min)),
    );
    child.shrink_clip_rect(rect);
    add(&mut child)
}

/// Shows `add` in a `size` area of `layout`, which it fills.
pub fn area<R>(
    ui: &mut Ui,
    size: Vec2,
    layout: Layout,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.allocate_ui_with_layout(size, layout, |ui| {
        ui.set_min_size(size);
        add(ui)
    })
}
