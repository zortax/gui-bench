//! The fine and shared variants: a `List` over the signal of a `Vec` of rows, and every row holds
//! its label, value and selection in signals of its own. An edit or a tick sets the signals of
//! the rows it changes and leaves the list alone; a structural operation sets the list.
//!
//! vizia's `List` has no keys: when its signal changes, it compares the new items with the old
//! ones by value and rebuilds every item from the first one that differs. The items here are
//! handles to the signals of a row, so two items are equal when they are the same row, which is
//! as close to keys as the list gets: rows before the first change stay, the rest are built anew.
//!
//! Selection: in the fine variant a click clears the `selected` signal of the old row and sets the
//! one of the new row, so a change wakes two rows. In the shared variant every row maps the one
//! selection signal to its class, so a change wakes every row.

use vizia::prelude::*;

use super::TableEvent;
use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids, Row, TableRow};

/// A row with its label, value and selection in signals.
#[derive(Clone, Copy, PartialEq)]
struct FineRow {
    id: u64,
    label: Signal<String>,
    value: Signal<u32>,
    selected: Signal<bool>,
}

impl TableRow for FineRow {
    fn from_row(row: Row) -> Self {
        Self {
            id: row.id,
            label: Signal::new(row.label),
            value: Signal::new(row.value),
            selected: Signal::new(false),
        }
    }

    fn toggle_suffix(&mut self) {
        self.label
            .update(|label| *label = spec::toggle_suffix(label));
    }
}

/// How the table holds the selection.
enum Selection {
    /// The selected row, whose `selected` signal is set (fine).
    Row(Option<FineRow>),
    /// The id of the selected row in one signal that every row reads (shared).
    Shared(Signal<Option<u64>>),
}

/// The events of the rows.
enum FineEvent {
    /// A row was clicked.
    Select(FineRow),
}

/// The state of the table.
struct FineTable {
    ids: Ids,
    count: usize,
    updates: usize,
    rows: Signal<Vec<FineRow>>,
    tick: Signal<u64>,
    selection: Selection,
}

impl Model for FineTable {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|table_event, _| match *table_event {
            TableEvent::Run(op) if op.changes_structure() => {
                self.rows
                    .update(|rows| op.apply(rows, &mut self.ids, self.count));
            }
            // The partial update edits labels only, so the list itself does not change.
            TableEvent::Run(_) => self.rows.with(|rows| {
                for mut row in rows.iter().step_by(10).copied() {
                    row.toggle_suffix();
                }
            }),
            TableEvent::Tick => {
                let now = self.tick.get() + 1;
                self.tick.set(now);
                self.rows.with(|rows| {
                    for at in spec::ticker_positions(rows.len(), self.updates) {
                        let row = rows[at];
                        row.value.set(spec::ticker_value(row.id, now));
                    }
                });
            }
        });
        event.map(|FineEvent::Select(row), _| self.select(*row));
    }
}

impl FineTable {
    /// Selects `row`, or clears the selection when it is the selected row.
    fn select(&mut self, row: FineRow) {
        match &mut self.selection {
            Selection::Row(selected) => {
                if let Some(old) = selected.take() {
                    old.selected.set(false);
                    if old == row {
                        return;
                    }
                }
                row.selected.set(true);
                *selected = Some(row);
            }
            Selection::Shared(selected) => {
                selected.update(|s| {
                    *s = if *s == Some(row.id) {
                        None
                    } else {
                        Some(row.id)
                    }
                });
            }
        }
    }
}

/// Builds the table with per-row signals. With `shared`, every row reads one selection signal.
pub fn build(cx: &mut Context, count: usize, ticker: Option<Ticker>, shared: bool) {
    let mut ids = Ids::default();
    let rows = Signal::new(ids.rows_of::<FineRow>(count));
    let tick = Signal::new(0_u64);
    let shared_selection = shared.then(|| Signal::new(None::<u64>));
    FineTable {
        ids,
        count,
        updates: ticker.map_or(0, |ticker| ticker.updates),
        rows,
        tick,
        selection: match shared_selection {
            Some(selected) => Selection::Shared(selected),
            None => Selection::Row(None),
        },
    }
    .build(cx);

    let status = Memo::new(move |_| super::status(rows.with(Vec::len), ticker, tick.get()));
    page::build(
        cx,
        status,
        |_| {},
        move |cx| {
            List::new(cx, rows, move |cx, _, item| {
                table_row(cx, item.get(), shared_selection);
            })
            .show_horizontal_scrollbar(false)
            .class("list");
        },
    );
}

/// Builds one row. Its `selected` class follows its own signal, or the shared selection signal
/// when there is one. A click selects the row.
fn table_row(cx: &mut Context, row: FineRow, shared: Option<Signal<Option<u64>>>) {
    let id = row.id;
    let handle = HStack::new(cx, move |cx| {
        Label::new(cx, row.label).class("label").hoverable(false);
        Label::new(cx, row.value).class("value").hoverable(false);
    })
    .class("row")
    .on_press(move |cx| cx.emit(FineEvent::Select(row)));
    match shared {
        Some(selected) => handle.toggle_class("selected", selected.map(move |s| *s == Some(id))),
        None => handle.toggle_class("selected", row.selected),
    };
}
