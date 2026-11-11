//! Function keys: a global listener hears every key of the window, whatever has focus, and
//! turns F1 to F8 into table operations.

use vizia::prelude::*;

use super::TableEvent;
use crate::spec::Op;

/// Makes the window send `TableEvent::Run` for every press of F1 to F8.
pub fn listen(cx: &mut Context) {
    cx.add_global_listener(|cx, event| {
        event.map(|window_event: &WindowEvent, _| {
            if let WindowEvent::KeyDown(code, _) = window_event
                && let Some(op) = op_of(*code)
            {
                cx.emit(TableEvent::Run(op));
            }
        });
    });
}

/// Returns the table operation of a key, F1 to F8.
fn op_of(code: Code) -> Option<Op> {
    let n = match code {
        Code::F1 => 1,
        Code::F2 => 2,
        Code::F3 => 3,
        Code::F4 => 4,
        Code::F5 => 5,
        Code::F6 => 6,
        Code::F7 => 7,
        Code::F8 => 8,
        _ => return None,
    };
    Op::from_f_key(n)
}
