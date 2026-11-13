//! The messages page: one card per message in a plain `column`. The body is a `text` that fills
//! the card's width and wraps there, so the height of a card follows its text.

use std::borrow::Cow;

use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use super::sidebar::avatar;
use super::style::{self, SEMIBOLD, color};
use super::{Message, spec};

/// The space between the avatar and the text, and between the author and the time.
const AVATAR_GAP: f32 = spec::MESSAGE_TEXT_X - spec::MESSAGE_PADDING.1 - spec::MESSAGE_AVATAR;
const TIME_GAP: f32 = 8.0;

/// Returns the feed of message cards. The direct edition builds every card from the state on
/// each view; the tuned edition wraps the feed in iced's `lazy`, keyed by the number of
/// messages, since the messages never change.
pub fn view(messages: &[spec::Message]) -> Element<'_, Message> {
    #[cfg(feature = "tuned")]
    return iced::widget::lazy(messages.len(), move |_| {
        feed(messages.iter().cloned().map(Cow::Owned))
    })
    .into();
    #[cfg(not(feature = "tuned"))]
    feed(messages.iter().map(Cow::Borrowed))
}

/// Returns the column of cards.
fn feed<'a>(messages: impl Iterator<Item = Cow<'a, spec::Message>>) -> Element<'a, Message> {
    column(messages.map(card)).spacing(spec::MESSAGE_GAP).into()
}

/// Returns one card: the avatar, the author and the time on one line, and the body below.
fn card<'a>(message: Cow<'a, spec::Message>) -> Element<'a, Message> {
    let avatar_color = message.avatar;
    let [initials, author, time, body]: [Cow<'a, str>; 4] = match message {
        Cow::Borrowed(message) => [
            &message.initials,
            &message.author,
            &message.time,
            &message.body,
        ]
        .map(|text| Cow::Borrowed(text.as_str())),
        Cow::Owned(message) => {
            [message.initials, message.author, message.time, message.body].map(Cow::Owned)
        }
    };
    let line = text::LineHeight::Absolute(spec::MESSAGE_LINE.into());
    let head = row![
        text(author)
            .size(spec::FONT_SIZE)
            .font(SEMIBOLD)
            .line_height(line)
            .color(color(spec::TEXT)),
        text(time)
            .size(13)
            .line_height(line)
            .color(color(spec::MUTED)),
    ]
    .spacing(TIME_GAP);
    let body = text(body)
        .size(spec::FONT_SIZE)
        .line_height(line)
        .color(color(spec::TEXT))
        .width(Length::Fill);
    container(
        row![
            avatar(initials, spec::MESSAGE_AVATAR, 13.0, avatar_color),
            column![head, body].width(Length::Fill),
        ]
        .spacing(AVATAR_GAP),
    )
    .padding([spec::MESSAGE_PADDING.0, spec::MESSAGE_PADDING.1])
    .width(Length::Fill)
    .style(style::card)
    .into()
}
