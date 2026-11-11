//! Function keys: the table hears the window's keyboard input through dioxus-native's window event
//! hook, so the keys work while nothing has focus, and each one names a table operation.
//!
//! A DOM `onkeydown` would not do: Blitz sends key events to the focused element and, with nothing
//! focused, to the `<html>` element, which is outside the Dioxus tree.

use dioxus_native::use_window_event;
use dioxus_native::winit::event::{ElementState, WindowEvent};
use dioxus_native::winit::keyboard::{Key, NamedKey};

use crate::spec::Op;

/// Returns the table operation of a key, F1 to F8.
fn op_of(key: &Key) -> Option<Op> {
    let n = match key {
        Key::Named(NamedKey::F1) => 1,
        Key::Named(NamedKey::F2) => 2,
        Key::Named(NamedKey::F3) => 3,
        Key::Named(NamedKey::F4) => 4,
        Key::Named(NamedKey::F5) => 5,
        Key::Named(NamedKey::F6) => 6,
        Key::Named(NamedKey::F7) => 7,
        Key::Named(NamedKey::F8) => 8,
        _ => return None,
    };
    Op::from_f_key(n)
}

/// Calls `run` with the operation of every function key that the window gets, for as long as the
/// current component lives.
pub fn use_function_keys(mut run: impl FnMut(Op) + 'static) {
    use_window_event(move |event, _| {
        if let WindowEvent::KeyboardInput { event, .. } = event
            && event.state == ElementState::Pressed
            && let Some(op) = op_of(&event.logical_key)
        {
            run(op);
        }
    });
}
