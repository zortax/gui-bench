//! The rows window: a borderless eframe window in the light theme with one vertical scroll area
//! that lays out all N rows every frame and scrolls 1:1 with the input.

use eframe::egui::{
    self, Context, RawInput, Stroke, TextStyle, Theme, Ui, style::ScrollStyle, vec2,
};

use super::row;
use crate::scroll::Scroll;
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

/// The rows view and its scroll state.
struct Rows {
    count: usize,
    scroll: Scroll,
}

impl Rows {
    fn new(count: usize) -> Self {
        Self {
            count,
            scroll: Scroll::default(),
        }
    }
}

impl eframe::App for Rows {
    fn raw_input_hook(&mut self, _: &Context, input: &mut RawInput) {
        self.scroll.take(input);
    }

    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        let output = self.scroll.area().show(ui, |ui| {
            for i in 0..self.count {
                row::row(ui, i);
            }
        });
        self.scroll.end(&output);
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        BACKGROUND.to_normalized_gamma_f32()
    }
}
