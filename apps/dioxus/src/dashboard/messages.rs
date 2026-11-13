//! The messages page: one card per message, whose body wraps to the width of the card.

use dioxus::prelude::*;

use super::css_color;
use super::spec::{self, Message};

/// The feed of `count` message cards.
#[component]
pub fn Messages(count: usize) -> Element {
    rsx! {
        div { class: "content__body feed",
            for (index , message) in spec::messages(count).into_iter().enumerate() {
                MessageCard { key: "{index}", message }
            }
        }
    }
}

/// One message: the avatar, the author and the time on one line, and the body below.
#[component]
fn MessageCard(message: Message) -> Element {
    rsx! {
        div { class: "card message",
            span {
                class: "message__avatar",
                background_color: css_color(message.avatar),
                "{message.initials}"
            }
            div { class: "message__text",
                div { class: "message__head",
                    span { class: "message__author", "{message.author}" }
                    span { class: "message__time", "{message.time}" }
                }
                div { class: "message__body", "{message.body}" }
            }
        }
    }
}
