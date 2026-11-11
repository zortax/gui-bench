//! The fine and shared variants: the rows are a `KeyedFields` collection with one lens per field,
//! the order is a signal of row ids, and a keyed `for` shows them. A row reads its label and value
//! by its id, so an edit of one field re-runs only the rows that show it, and the field signals
//! exist only while a row reads them. The fine variant marks the selected row through a
//! `Selector`, so a selection change wakes two rows; the shared variant has every row compare the
//! one selection signal.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use zgui::prelude::*;
use zgui::reactive::{KeyedFields, Lens, LocalStorage, Selector};

use super::keys;
use crate::cli::Ticker;
use crate::page::HeaderProps;
use crate::spec::{self, Ids, Op, Row, TableRow};

/// The fields of one row.
struct Item {
    label: String,
    value: u32,
}

const LABEL: Lens<Item, String> = Lens::new(0, |i| &i.label, |i| &mut i.label);
const VALUE: Lens<Item, u32> = Lens::new(1, |i| &i.value, |i| &mut i.value);

/// A row while an operation runs on the order: its id, the record of a new row, and whether the
/// operation toggled its label.
struct Slot {
    id: u64,
    new: Option<Row>,
    toggled: bool,
}

impl TableRow for Slot {
    fn from_row(row: Row) -> Self {
        Self {
            id: row.id,
            new: Some(row),
            toggled: false,
        }
    }

    fn toggle_suffix(&mut self) {
        self.toggled = !self.toggled;
    }
}

fn item(row: Row) -> (u64, Item) {
    (
        row.id,
        Item {
            label: row.label,
            value: row.value,
        },
    )
}

/// Runs an operation: the order changes when the operation moves rows, and the collection gets
/// the new records, loses the removed ones and changes the toggled labels.
fn run_op(
    op: Op,
    order: RwSignal<Vec<u64>>,
    table: KeyedFields<u64, Item>,
    ids: &mut Ids,
    count: usize,
) {
    let mut slots: Vec<Slot> = order.with_untracked(|order| {
        order
            .iter()
            .map(|&id| Slot {
                id,
                new: None,
                toggled: false,
            })
            .collect()
    });
    op.apply(&mut slots, ids, count);
    for slot in &mut slots {
        if let Some(row) = slot.new.take() {
            let (id, record) = item(row);
            table.insert(id, record);
        }
        if slot.toggled {
            table.update(&slot.id, LABEL, |label| *label = spec::toggle_suffix(label));
        }
    }
    if op.changes_structure() {
        let kept: HashSet<u64> = slots.iter().map(|slot| slot.id).collect();
        order.with_untracked(|old| {
            for id in old.iter().filter(|id| !kept.contains(id)) {
                table.remove(id);
            }
        });
        order.set(slots.into_iter().map(|slot| slot.id).collect());
    }
}

/// The table with per-field signals. With `shared`, every row reads the selection signal.
#[component]
pub fn FineTable(count: usize, ticker: Option<Ticker>, shared: bool) -> impl IntoView {
    let ids = Rc::new(RefCell::new(Ids::default()));
    let rows = ids.borrow_mut().rows(count);
    let order = RwSignal::new(rows.iter().map(|row| row.id).collect::<Vec<_>>());
    let table = KeyedFields::from_values(rows.into_iter().map(item));
    let selected = RwSignal::new(None::<u64>);
    let selector = Selector::new(move || selected.get());
    let tick = RwSignal::new(0_u64);
    let _timer = RwSignal::new_local(ticker.map(|ticker| {
        set_interval(Duration::from_millis(ticker.interval_ms), move || {
            let now = tick.get_untracked() + 1;
            tick.set(now);
            order.with_untracked(|order| {
                for at in spec::ticker_positions(order.len(), ticker.updates) {
                    let id = order[at];
                    table.set(&id, VALUE, spec::ticker_value(id, now));
                }
            });
        })
    }));
    let status = Signal::derive(move || {
        let len = order.with(Vec::len);
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
                    run_op(op, order, table, &mut ids.borrow_mut(), count);
                }
            }
        ) {
            Header(status = status) {""}
            scroll(class = "list") {
                for id in move || order.get(), key = |id: &u64| *id {
                    {table_row(id, table, selected, &selector, shared)}
                }
            }
        }
    }
}

/// Returns one row, with its selection read through `selector`, or through the shared selection
/// signal when `shared`.
fn table_row(
    id: u64,
    table: KeyedFields<u64, Item>,
    selected: RwSignal<Option<u64>>,
    selector: &Selector<Option<u64>>,
    shared: bool,
) -> impl IntoView {
    let selector = selector.clone();
    let is_selected = Signal::derive_local(move || {
        if shared {
            selected.get() == Some(id)
        } else {
            selector.is_selected(&Some(id))
        }
    });
    view! { TableRow(id = id, table = table, selected = selected, is_selected = is_selected) }
}

/// One row: a click selects it, or clears the selection when it is the selected one.
#[component]
fn TableRow(
    id: u64,
    table: KeyedFields<u64, Item>,
    selected: RwSignal<Option<u64>>,
    is_selected: Signal<bool, LocalStorage>,
) -> impl IntoView {
    view! {
        row(
            class = "row",
            class:selected = is_selected,
            on:click = move |_| selected.update(|s| *s = if *s == Some(id) { None } else { Some(id) })
        ) {
            text(class = "row__label") {{move || table.get(&id, LABEL).unwrap_or_default()}}
            text(class = "row__value") {{move || table.get(&id, VALUE).unwrap_or_default().to_string()}}
        }
    }
}
