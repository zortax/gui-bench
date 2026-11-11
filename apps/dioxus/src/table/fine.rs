//! The fine and shared variants: the row records live in a `Store` of a `HashMap` keyed by row
//! id (dioxus-stores), the order is a signal of ids, and the table renders one keyed `FineRow`
//! component per id. A row reads its label and value through the store scoped to its id, so an
//! edit of one field renders only the row that shows it; a structural operation changes the order,
//! and the keyed diff keeps every row component whose props stay. Removed records leave the map,
//! so nothing outlives its row.
//!
//! Selection: in the fine variant the table reads the selection and hands each row an
//! `is_selected` prop, so a selection change renders the table, and of the rows only the two
//! whose prop changed (Dioxus skips components whose props are equal). In the shared variant
//! every row reads the one selection signal itself, so a selection change renders every row.

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;

use super::header::TableHeader;
use super::keys::use_function_keys;
use super::ticker::use_ticker;
use super::toggle_selection;
use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids, Op, Row, TableRow};

/// The fields of one row. The derive gives the store a `label` and a `value` scope.
#[derive(Store)]
struct Item {
    label: String,
    value: u32,
}

/// The records of all rows by id.
type Items = Store<HashMap<u64, Item>>;

impl From<Row> for Item {
    fn from(row: Row) -> Self {
        Self {
            label: row.label,
            value: row.value,
        }
    }
}

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

/// Runs an operation: the store gets the new records and the toggled labels, and when the
/// operation moves rows, loses the removed records and the order changes.
fn run_op(op: Op, mut order: Signal<Vec<u64>>, mut items: Items, ids: &mut Ids, count: usize) {
    let mut slots: Vec<Slot> = order
        .peek()
        .iter()
        .map(|&id| Slot {
            id,
            new: None,
            toggled: false,
        })
        .collect();
    op.apply(&mut slots, ids, count);
    for slot in &mut slots {
        if let Some(row) = slot.new.take() {
            items.insert(row.id, row.into());
        }
        if slot.toggled {
            items
                .get_unchecked(slot.id)
                .label()
                .with_mut(|label| *label = spec::toggle_suffix(label));
        }
    }
    if op.changes_structure() {
        let kept: HashSet<u64> = slots.iter().map(|slot| slot.id).collect();
        items.retain(|id, _| kept.contains(id));
        order.set(slots.into_iter().map(|slot| slot.id).collect());
    }
}

/// The table with per-row store scopes. With `shared`, every row reads the selection signal.
#[component]
pub fn FineTable(count: usize, ticker: Option<Ticker>, shared: bool) -> Element {
    let mut ids = use_signal(Ids::default);
    let mut items: Items = use_store(HashMap::new);
    let order = use_signal(|| {
        let rows = ids.write().rows(count);
        let order = rows.iter().map(|row| row.id).collect::<Vec<_>>();
        items
            .write()
            .extend(rows.into_iter().map(|row| (row.id, row.into())));
        order
    });
    let selected = use_signal(|| None::<u64>);
    let tick = use_ticker(ticker, move |now, updates| {
        let order = order.peek();
        for at in spec::ticker_positions(order.len(), updates) {
            let id = order[at];
            items
                .get_unchecked(id)
                .value()
                .set(spec::ticker_value(id, now));
        }
    });
    use_function_keys(move |op| run_op(op, order, items, &mut ids.write(), count));

    // Only the fine variant reads the selection here; shared rows read it themselves.
    let current = if shared { None } else { selected() };
    rsx! {
        style { {page::CSS} }
        div { class: "page",
            TableHeader { rows: order, tick, ticker: ticker.is_some() }
            div { class: "list",
                for &id in order.read().iter() {
                    FineRow {
                        key: "{id}",
                        id,
                        items,
                        selected,
                        is_selected: (!shared).then_some(current == Some(id)),
                    }
                }
            }
        }
    }
}

/// One row: a click selects it, or clears the selection when it is the selected one. With
/// `is_selected` the parent says whether it is selected; without, the row reads `selected`.
///
/// The row scopes the store to its id with `get_unchecked`: `get` would also track the keys of
/// the map, so every insert and remove would render every row again.
#[component]
fn FineRow(
    id: u64,
    items: Items,
    selected: Signal<Option<u64>>,
    #[props(!optional)] is_selected: Option<bool>,
) -> Element {
    let item = items.get_unchecked(id);
    let is_selected = is_selected.unwrap_or_else(|| selected() == Some(id));
    rsx! {
        div {
            class: "row",
            class: if is_selected { "selected" },
            onclick: move |_| toggle_selection(selected, id),
            span { class: "row__label", "{item.label()}" }
            span { class: "row__value", "{item.value()}" }
        }
    }
}
