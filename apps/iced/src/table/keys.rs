//! Function keys: the application hears the key presses that no widget took, and each of F1 to
//! F8 names a table operation.

use iced::Subscription;
use iced::keyboard::{self, Key, key::Named};

use super::Message;
use crate::spec::Op;

/// Returns the subscription that turns F1 to F8 into table operations.
pub fn listen() -> Subscription<Message> {
    keyboard::listen().filter_map(op_of)
}

/// Returns the table operation of a key press, F1 to F8.
fn op_of(event: keyboard::Event) -> Option<Message> {
    let keyboard::Event::KeyPressed {
        key: Key::Named(key),
        ..
    } = event
    else {
        return None;
    };
    let n = match key {
        Named::F1 => 1,
        Named::F2 => 2,
        Named::F3 => 3,
        Named::F4 => 4,
        Named::F5 => 5,
        Named::F6 => 6,
        Named::F7 => 7,
        Named::F8 => 8,
        _ => return None,
    };
    Op::from_f_key(n).map(Message::Key)
}
