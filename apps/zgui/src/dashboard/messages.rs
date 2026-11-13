//! The messages page: one card per message, whose body wraps to the width of the card.

use zgui::prelude::*;

use super::hex;
use super::spec::{self, Message};

/// The feed of `count` message cards. The messages never change, so a plain list holds them.
#[component]
pub fn MessageFeed(count: usize) -> impl IntoView {
    view! {
        column(class = "content__body feed") {
            {spec::messages(count)
                .into_iter()
                .map(|message| view! { MessageCard(message = message) }.into_view())
                .collect::<Vec<_>>()}
        }
    }
}

/// One message: the avatar, the author and the time on one line, and the body below.
#[component]
fn MessageCard(message: Message) -> impl IntoView {
    view! {
        row(class = "card message") {
            row(class = "message__avatar", style:background-color = hex(message.avatar)) {
                {message.initials}
            }
            column(class = "message__text") {
                row(class = "message__head") {
                    text(class = "message__author") {{message.author}}
                    text(class = "message__time") {{message.time}}
                }
                text(class = "message__body") {{message.body}}
            }
        }
    }
}
