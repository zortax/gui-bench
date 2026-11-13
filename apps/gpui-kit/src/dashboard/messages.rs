//! The messages page: one card per message, whose body wraps to the width of the card.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::*;

use super::spec::{self, Message};
use super::style::card;

/// One message with its texts.
struct Card {
    avatar: u32,
    initials: SharedString,
    author: SharedString,
    time: SharedString,
    body: SharedString,
}

impl Card {
    fn new(message: Message) -> Self {
        Self {
            avatar: message.avatar,
            initials: message.initials.into(),
            author: message.author.into(),
            time: message.time.into(),
            body: message.body.into(),
        }
    }

    /// Returns the card: the avatar, the author and the time on one line, and the body below.
    fn render(&self) -> Div {
        let head = h_flex()
            .items_baseline()
            .gap(px(8.0))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.author.clone()),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(rgb(spec::MUTED))
                    .child(self.time.clone()),
            );
        card()
            .flex()
            .flex_none()
            .w_full()
            .py(px(spec::MESSAGE_PADDING.0))
            .px(px(spec::MESSAGE_PADDING.1))
            .gap(px(spec::MESSAGE_TEXT_X - spec::MESSAGE_PADDING.1 - spec::MESSAGE_AVATAR))
            .child(
                h_flex()
                    .flex_none()
                    .size(px(spec::MESSAGE_AVATAR))
                    .justify_center()
                    .rounded_full()
                    .bg(rgb(self.avatar))
                    .text_size(px(13.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0xffffff))
                    .child(self.initials.clone()),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .line_height(px(spec::MESSAGE_LINE))
                    .child(head)
                    .child(div().w_full().child(self.body.clone())),
            )
    }
}

/// The feed of message cards.
pub struct Messages {
    cards: Vec<Card>,
}

impl Messages {
    pub fn new(count: usize) -> Self {
        Self {
            cards: spec::messages(count).into_iter().map(Card::new).collect(),
        }
    }
}

impl Render for Messages {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .p(px(spec::CONTENT_PADDING))
            .gap(px(spec::MESSAGE_GAP))
            .children(self.cards.iter().map(Card::render))
    }
}
