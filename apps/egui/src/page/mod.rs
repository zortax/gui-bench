//! The page of the header scenarios: a top panel with the status text and a slot on the right,
//! and a vertical scroll area of rows below it. Rows here have one background and no hover color.

mod header;
mod row;

use eframe::egui::{Color32, Context, Stroke, TextStyle, Theme, Ui, style::ScrollStyle, vec2};

pub use header::header;
pub use row::row;

use crate::scroll::Scroll;
use crate::{font, spec};

/// Returns the egui color of a spec color.
pub const fn color(color: u32) -> Color32 {
    let (r, g, b) = spec::rgb(color);
    Color32::from_rgb(r, g, b)
}

/// The window background behind the page.
pub const BACKGROUND: Color32 = color(spec::BACKGROUND);

/// Sets the light theme, the system font at 14 px, the header separator color, solid scroll
/// bars, no item spacing, no animation, and labels that do not select text (so a click on a label
/// goes to its row). The text cursor keeps its default blink, as in every app.
pub fn setup(ctx: &Context) {
    ctx.set_theme(Theme::Light);
    ctx.set_fonts(font::definitions());
    ctx.all_styles_mut(|style| {
        style.animation_time = 0.0;
        style.spacing.item_spacing = vec2(0.0, 0.0);
        style.spacing.scroll = ScrollStyle::solid();
        style.interaction.selectable_labels = false;
        style.visuals.widgets.noninteractive.bg_stroke =
            Stroke::new(spec::BORDER_HEIGHT, color(spec::HEADER_BORDER));
        if let Some(body) = style.text_styles.get_mut(&TextStyle::Body) {
            body.size = spec::FONT_SIZE;
        }
    });
}

/// Shows the rows that `add_rows` adds in the scroll area below the header. The row separators
/// take the row border color.
pub fn list(ui: &mut Ui, scroll: &mut Scroll, add_rows: impl FnOnce(&mut Ui)) {
    let output = scroll.area().show(ui, |ui| {
        ui.visuals_mut().widgets.noninteractive.bg_stroke =
            Stroke::new(spec::BORDER_HEIGHT, color(spec::ROW_BORDER));
        add_rows(ui);
    });
    scroll.end(&output);
}
