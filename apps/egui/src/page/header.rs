//! The header: a top panel of exactly 40 px, its separator line included, with the status text
//! on the left and a slot on the right.

use eframe::egui::{Align, Frame, Layout, Margin, Panel, RichText, Ui};

use super::color;
use crate::spec;

/// Shows the header with `status` on the left and what `add_right` adds at the right end.
pub fn header(ui: &mut Ui, status: &str, add_right: impl FnOnce(&mut Ui)) {
    let frame = Frame::new()
        .fill(color(spec::HEADER_BG))
        .inner_margin(Margin::symmetric(spec::PADDING_X as i8, 0));
    Panel::top("header")
        .exact_size(spec::HEADER_HEIGHT)
        .frame(frame)
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.label(RichText::new(status).color(color(spec::LABEL_COLOR)));
                ui.with_layout(Layout::right_to_left(Align::Center), add_right);
            });
        });
}
