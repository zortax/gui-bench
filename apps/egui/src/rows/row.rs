//! One row, built the way egui code usually builds it: a frame with a horizontal layout of two
//! labels, and a separator under it. The frame fill turns blue while the pointer is over it.

use eframe::egui::{Align, Color32, Frame, Layout, Margin, RichText, Separator, Ui, vec2};

/// The height of a row in logical pixels, its bottom separator included.
pub const ROW_HEIGHT: f32 = 32.0;
/// The height of the separator under each row.
const SEPARATOR_HEIGHT: f32 = 1.0;
const PADDING_X: i8 = 12;

const ROW_EVEN: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
const ROW_ODD: Color32 = Color32::from_rgb(0xf6, 0xf8, 0xfa);
const ROW_HOVER: Color32 = Color32::from_rgb(0xcf, 0xe3, 0xff);
const LABEL_COLOR: Color32 = Color32::from_rgb(0x1f, 0x23, 0x28);
const VALUE_COLOR: Color32 = Color32::from_rgb(0x6e, 0x77, 0x81);

/// The color of the separator between rows.
pub const ROW_BORDER: Color32 = Color32::from_rgb(0xe5, 0xe7, 0xeb);

/// Adds row `i`: the frame follows the hover pattern of the `Frame` documentation.
pub fn row(ui: &mut Ui, i: usize) {
    let fill = if i.is_multiple_of(2) {
        ROW_EVEN
    } else {
        ROW_ODD
    };
    let mut frame = Frame::new()
        .fill(fill)
        .inner_margin(Margin::symmetric(PADDING_X, 0))
        .begin(ui);
    {
        let ui = &mut frame.content_ui;
        let size = vec2(ui.available_width(), ROW_HEIGHT - SEPARATOR_HEIGHT);
        ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
            ui.set_min_size(size);
            ui.label(RichText::new(format!("Row {i:05}")).color(LABEL_COLOR));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(((i * 7919) % 10000).to_string()).color(VALUE_COLOR));
            });
        });
    }
    let response = frame.allocate_space(ui);
    if response.hovered() {
        frame.frame.fill = ROW_HOVER;
    }
    frame.paint(ui);
    ui.add(Separator::default().spacing(SEPARATOR_HEIGHT));
}
