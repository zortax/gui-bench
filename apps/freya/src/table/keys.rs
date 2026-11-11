//! Function keys: the root listens to them globally, so they work while nothing has focus, and
//! each one names a table operation.

use freya::prelude::*;

use crate::spec::Op;

/// Returns the table operation of a key, F1 to F8.
pub fn op_of(key: &Key) -> Option<Op> {
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
