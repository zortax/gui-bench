//! The messages page: one card per message, whose body wraps to the width of the card.

use eframe::egui::{Align, Label, Layout, Margin, Ui, vec2};

use super::spec::{self, Message};
use super::style::{Weight, area, avatar, card, text};

/// The space between the avatar and the text.
const TEXT_GAP: f32 = spec::MESSAGE_TEXT_X - spec::MESSAGE_PADDING.1 - spec::MESSAGE_AVATAR;
/// The space between the author and the time.
const TIME_GAP: f32 = 8.0;

/// Shows the feed of `messages`, all of them laid out every pass.
pub fn show(ui: &mut Ui, messages: &[Message]) {
    for (i, message) in messages.iter().enumerate() {
        if i > 0 {
            ui.add_space(spec::MESSAGE_GAP);
        }
        message_card(ui, message);
    }
}

/// One message: the avatar, the author and the time on one line, and the body below.
fn message_card(ui: &mut Ui, message: &Message) {
    let (down, across) = spec::MESSAGE_PADDING;
    card()
        .inner_margin(Margin::symmetric(across as i8, down as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let initials = text(&message.initials, 13.0, Weight::Semibold, 0xffffff);
                avatar(ui, spec::MESSAGE_AVATAR, message.avatar, initials);
                ui.add_space(TEXT_GAP);
                ui.vertical(|ui| {
                    let head = vec2(0.0, spec::MESSAGE_LINE);
                    area(ui, head, Layout::left_to_right(Align::Center), |ui| {
                        let author = text(
                            &message.author,
                            spec::FONT_SIZE,
                            Weight::Semibold,
                            spec::TEXT,
                        );
                        ui.label(author);
                        ui.add_space(TIME_GAP);
                        ui.label(text(&message.time, 13.0, Weight::Regular, spec::MUTED));
                    });
                    let body = text(&message.body, spec::FONT_SIZE, Weight::Regular, spec::TEXT)
                        .line_height(Some(spec::MESSAGE_LINE));
                    ui.add(Label::new(body).wrap());
                });
            });
        });
}
