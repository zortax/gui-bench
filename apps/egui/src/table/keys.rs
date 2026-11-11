//! Function keys: the app takes F1 to F8 out of the input of the pass, and each one names a
//! table operation.

use eframe::egui::{Key, Modifiers, Ui};

use crate::spec::Op;

const KEYS: [Key; 8] = [
    Key::F1,
    Key::F2,
    Key::F3,
    Key::F4,
    Key::F5,
    Key::F6,
    Key::F7,
    Key::F8,
];

/// Consumes the function key presses of this pass and returns their operations in key order.
pub fn take_ops(ui: &Ui) -> Vec<Op> {
    ui.input_mut(|input| {
        KEYS.iter()
            .zip(Op::ALL)
            .filter(|(key, _)| input.consume_key(Modifiers::NONE, **key))
            .map(|(_, op)| op)
            .collect()
    })
}
