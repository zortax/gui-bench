//! A row of the header scenarios, built like the rows scenario builds its rows: a frame with a
//! horizontal layout of two labels, and a separator under it. The row senses clicks.

use eframe::egui::{Align, Frame, Layout, Margin, Response, RichText, Sense, Separator, Ui, vec2};

use super::color;
use crate::spec::{self, Row};

/// Adds `row` with the selected background when `selected`, and returns its response, which
/// reports clicks. The row's widgets take their ids from the row id.
pub fn row(ui: &mut Ui, row: &Row, selected: bool) -> Response {
    let fill = color(if selected {
        spec::SELECTED
    } else {
        spec::ROW_EVEN
    });
    ui.push_id(row.id, |ui| {
        let response = Frame::new()
            .fill(fill)
            .inner_margin(Margin::symmetric(spec::PADDING_X as i8, 0))
            .show(ui, |ui| {
                let size = vec2(ui.available_width(), spec::ROW_HEIGHT - spec::BORDER_HEIGHT);
                ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                    ui.set_min_size(size);
                    ui.label(RichText::new(&row.label).color(color(spec::LABEL_COLOR)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(row.value.to_string()).color(color(spec::VALUE_COLOR)),
                        );
                    });
                });
            })
            .response
            .interact(Sense::click());
        ui.add(Separator::default().spacing(spec::BORDER_HEIGHT));
        response
    })
    .inner
}
