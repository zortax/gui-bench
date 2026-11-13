//! The messages page: one card per message, whose body wraps to the width of the card.

use floem::prelude::*;

use super::style::{self, Weight, circle, px, text};
use crate::dash::{self, Message};

/// Returns the feed of `count` message cards. The messages never change, so a plain stack holds
/// them.
pub fn view(count: usize) -> impl IntoView {
    Stack::vertical_from_iter(dash::messages(count).into_iter().map(card))
        .style(|s| super::body(s).row_gap(px(dash::MESSAGE_GAP)))
}

/// One message: the avatar, the author and the time on one line, and the body below.
fn card(message: Message) -> impl IntoView {
    let avatar = Container::new(text(&message.initials, 13.0, Weight::Semibold, 0xffffff))
        .style(move |s| circle(s, dash::MESSAGE_AVATAR, message.avatar));
    let author = text(&message.author, dash::FONT_SIZE, Weight::Semibold, dash::TEXT);
    let time = text(&message.time, 13.0, Weight::Normal, dash::MUTED);
    let head = Stack::horizontal((author, time)).style(|s| {
        s.height(px(dash::MESSAGE_LINE))
            .col_gap(8.0)
            .items_center()
    });
    let body = text(&message.body, dash::FONT_SIZE, Weight::Normal, dash::TEXT).style(|s| {
        s.width_full()
            .text_wrap()
            .line_height(px(dash::MESSAGE_LINE).pt())
    });
    let texts = Stack::vertical((head, body)).style(|s| s.flex_grow(1.0_f32).flex_basis(0.0).min_width(0.0));
    Stack::horizontal((avatar, texts)).style(|s| {
        style::card(s)
            .width_full()
            .padding_vert(px(dash::MESSAGE_PADDING.0))
            .padding_horiz(px(dash::MESSAGE_PADDING.1))
            .col_gap(px(
                dash::MESSAGE_TEXT_X - dash::MESSAGE_PADDING.1 - dash::MESSAGE_AVATAR,
            ))
            .items_start()
    })
}
