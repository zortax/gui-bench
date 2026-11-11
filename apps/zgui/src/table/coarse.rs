//! The coarse variant: one signal holds every row, and the list is one reactive hole over it and
//! the selection. Any change, an edit of one row or a new selection included, runs the hole again
//! and builds every row anew.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use zgui::prelude::*;

use super::keys;
use crate::cli::Ticker;
use crate::page::HeaderProps;
use crate::spec::{self, Ids, Row};

/// The table with one signal for all rows.
#[component]
pub fn CoarseTable(count: usize, ticker: Option<Ticker>) -> impl IntoView {
    let ids = Rc::new(RefCell::new(Ids::default()));
    let rows = RwSignal::new(ids.borrow_mut().rows(count));
    let selected = RwSignal::new(None::<u64>);
    let tick = RwSignal::new(0_u64);
    let _timer = RwSignal::new_local(ticker.map(|ticker| {
        set_interval(Duration::from_millis(ticker.interval_ms), move || {
            let now = tick.get_untracked() + 1;
            tick.set(now);
            rows.update(|rows| {
                for at in spec::ticker_positions(rows.len(), ticker.updates) {
                    rows[at].value = spec::ticker_value(rows[at].id, now);
                }
            });
        })
    }));
    let status = Signal::derive(move || {
        let len = rows.with(Vec::len);
        match ticker {
            Some(_) => spec::ticker_status(len, tick.get()),
            None => spec::status(len),
        }
    });
    let anchor = NodeRef::new();
    keys::window_keys(anchor);

    view! {
        column(
            class = "page",
            node_ref = anchor,
            on:key_down = move |ev| {
                if let Some(op) = keys::op_of(&ev.key) {
                    rows.update(|rows| op.apply(rows, &mut ids.borrow_mut(), count));
                }
            }
        ) {
            Header(status = status) {""}
            scroll(class = "list") {
                {move || {
                    let current = selected.get();
                    rows.with(|rows| {
                        rows.iter()
                            .map(|row| AnyView::new(table_row(row.clone(), current == Some(row.id), selected)))
                            .collect::<Vec<_>>()
                    })
                }}
            }
        }
    }
}

/// Builds one row: a click selects it, or clears the selection when it is the selected one.
fn table_row(row: Row, is_selected: bool, selected: RwSignal<Option<u64>>) -> impl IntoView {
    let id = row.id;
    view! {
        row(
            class = "row",
            class:selected = is_selected,
            on:click = move |_| selected.update(|s| *s = if *s == Some(id) { None } else { Some(id) })
        ) {
            text(class = "row__label") {{row.label}}
            text(class = "row__value") {{row.value.to_string()}}
        }
    }
}
