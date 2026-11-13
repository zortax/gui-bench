//! The messages page: one card per message, whose body is a label that wraps to the width of the
//! card, so the height of the card follows its text.

use vizia::prelude::*;

use super::rgb;
use crate::dashboard_spec::{Message, messages};

/// Builds the feed of `count` message cards.
pub fn build(cx: &mut Context, count: usize) {
    VStack::new(cx, move |cx| {
        for message in messages(count) {
            card(cx, message);
        }
    })
    .class("blocks")
    .class("feed");
}

/// Builds one message: the avatar, the author and the time on one line, and the body below.
fn card(cx: &mut Context, message: Message) {
    let Message {
        author,
        initials,
        avatar,
        time,
        body,
    } = message;
    HStack::new(cx, move |cx| {
        HStack::new(cx, move |cx| {
            Label::new(cx, initials);
        })
        .class("message-avatar")
        .background_color(rgb(avatar));
        VStack::new(cx, move |cx| {
            HStack::new(cx, move |cx| {
                Label::new(cx, author).class("message-author");
                Label::new(cx, time).class("message-time");
            })
            .class("message-head");
            Label::new(cx, body).class("message-body");
        })
        .class("message-text");
    })
    .class("card")
    .class("message");
}
