//! Function keys: one action per table operation, bound to F1 to F8 in the table's key context.
//! The table root holds the focus, so the keys reach it while no other element has focus.

use gpui_kit::*;

use crate::spec::Op;

actions!(
    table,
    [
        Update10, Swap, Remove, Insert, Replace, Append, Truncate, Clear
    ]
);

/// The key context of the table root.
pub const CONTEXT: &str = "Table";

/// Binds F1 to F8 to the table actions.
pub fn bind(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("f1", Update10, Some(CONTEXT)),
        KeyBinding::new("f2", Swap, Some(CONTEXT)),
        KeyBinding::new("f3", Remove, Some(CONTEXT)),
        KeyBinding::new("f4", Insert, Some(CONTEXT)),
        KeyBinding::new("f5", Replace, Some(CONTEXT)),
        KeyBinding::new("f6", Append, Some(CONTEXT)),
        KeyBinding::new("f7", Truncate, Some(CONTEXT)),
        KeyBinding::new("f8", Clear, Some(CONTEXT)),
    ]);
}

/// Makes `root` the key context of the table and has every table action call `run` with its
/// operation.
pub fn on_ops<V: 'static>(
    root: Div,
    cx: &mut Context<V>,
    run: fn(&mut V, Op, &mut Context<V>),
) -> Div {
    root.key_context(CONTEXT)
        .on_action(cx.listener(move |v, _: &Update10, _, cx| run(v, Op::Update10, cx)))
        .on_action(cx.listener(move |v, _: &Swap, _, cx| run(v, Op::Swap, cx)))
        .on_action(cx.listener(move |v, _: &Remove, _, cx| run(v, Op::Remove, cx)))
        .on_action(cx.listener(move |v, _: &Insert, _, cx| run(v, Op::Insert, cx)))
        .on_action(cx.listener(move |v, _: &Replace, _, cx| run(v, Op::Replace, cx)))
        .on_action(cx.listener(move |v, _: &Append, _, cx| run(v, Op::Append, cx)))
        .on_action(cx.listener(move |v, _: &Truncate, _, cx| run(v, Op::Truncate, cx)))
        .on_action(cx.listener(move |v, _: &Clear, _, cx| run(v, Op::Clear, cx)))
}
