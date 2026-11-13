//! The messages page: one card per message, whose body wraps to the width of the card.

use freya::prelude::*;

use super::{avatar, card, color, spec, text};

/// The feed of `count` message cards. It has no state, so it renders once.
#[derive(PartialEq)]
pub struct Messages {
    pub count: usize,
}

impl Component for Messages {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::fill())
            .padding(spec::CONTENT_PADDING)
            .spacing(spec::MESSAGE_GAP)
            .children(spec::messages(self.count).into_iter().map(message_card))
    }
}

/// Returns the card of one message: the avatar, the author and the time on one line, and the
/// body below, whose lines set the height of the card.
fn message_card(message: spec::Message) -> Element {
    let (down, across) = spec::MESSAGE_PADDING;
    let head = rect()
        .horizontal()
        .height(Size::px(spec::MESSAGE_LINE))
        .spacing(8.0)
        .cross_align(Alignment::Center)
        .child(text(
            message.author,
            spec::FONT_SIZE,
            FontWeight::SEMI_BOLD,
            spec::TEXT,
        ))
        .child(text(message.time, 13.0, FontWeight::NORMAL, spec::MUTED));
    let body = label()
        .text(message.body)
        .width(Size::fill())
        .font_size(spec::FONT_SIZE)
        .line_height(spec::MESSAGE_LINE / spec::FONT_SIZE)
        .color(color(spec::TEXT));

    card()
        .horizontal()
        .width(Size::fill())
        .padding(Gaps::new(down, across, down, across))
        .spacing(spec::MESSAGE_TEXT_X - across - spec::MESSAGE_AVATAR)
        .child(avatar(
            message.initials,
            spec::MESSAGE_AVATAR,
            message.avatar,
            13.0,
        ))
        .child(rect().width(Size::fill()).child(head).child(body))
        .into()
}
