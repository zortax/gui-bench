//! The rows window: a borderless eframe window in the light theme with one vertical scroll area
//! that lays out all N rows every frame.

use eframe::egui::{
    self, Context, ScrollArea, Stroke, TextStyle, Theme, Ui, style::ScrollStyle, vec2,
};

use super::row;
use crate::{font, spec, window};

const BACKGROUND: egui::Color32 = egui::Color32::WHITE;

/// Opens the rows window with `count` rows and runs it until it closes.
pub fn run(title: &str, count: usize) -> eframe::Result {
    window::run(title, setup, move || Box::new(Rows::new(count)))
}

/// Sets the light theme, the system font at 14 px, the row separator color, solid scroll bars,
/// no item spacing and no animation.
pub fn setup(ctx: &Context) {
    ctx.set_theme(Theme::Light);
    ctx.set_fonts(font::definitions());
    ctx.all_styles_mut(|style| {
        style.animation_time = 0.0;
        style.spacing.item_spacing = vec2(0.0, 0.0);
        style.spacing.scroll = ScrollStyle::solid();
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, row::ROW_BORDER);
        if let Some(body) = style.text_styles.get_mut(&TextStyle::Body) {
            body.size = spec::FONT_SIZE;
        }
    });
}

/// The rows view.
struct Rows {
    count: usize,
}

impl Rows {
    fn new(count: usize) -> Self {
        Self { count }
    }
}

impl eframe::App for Rows {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            for i in 0..self.count {
                row::row(ui, i);
            }
        });
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        BACKGROUND.to_normalized_gamma_f32()
    }
}
