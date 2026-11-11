//! The fine and shared variants: a keyed `dyn_stack` over rows that each hold their label and
//! value in signals. An edit writes the signals of the rows it changes; a structural operation
//! changes the keyed list, which keeps the view of every row whose id stays. floem's reactive
//! crate has no selector, so the fine variant gives each row a selected flag signal and a click
//! writes the flags of the old and the new row only; the shared variant has every row's style
//! compare the one selection signal.

use std::time::Duration;

use floem::prelude::*;

use super::{keys, timer};
use crate::cli::Ticker;
use crate::page::{self, band, header, selected_style};
use crate::spec::{self, Ids, Row, TableRow};

/// A row with its label, value and selected flag in signals.
#[derive(Clone, Copy)]
struct FineRow {
    id: u64,
    label: RwSignal<String>,
    value: RwSignal<u32>,
    selected: RwSignal<bool>,
}

impl TableRow for FineRow {
    fn from_row(row: Row) -> Self {
        Self {
            id: row.id,
            label: RwSignal::new(row.label),
            value: RwSignal::new(row.value),
            selected: RwSignal::new(false),
        }
    }

    fn toggle_suffix(&mut self) {
        self.label
            .update(|label| *label = spec::toggle_suffix(label));
    }
}

/// How the rows learn the selection.
#[derive(Clone, Copy)]
enum Selection {
    /// The selected row, whose flag is set; no view reads this signal.
    Fine(RwSignal<Option<FineRow>>),
    /// The selected id, which every row reads.
    Shared(RwSignal<Option<u64>>),
}

impl Selection {
    /// Returns true when `row` is selected, and makes the caller track the state it reads.
    fn is_selected(self, row: FineRow) -> bool {
        match self {
            Self::Fine(_) => row.selected.get(),
            Self::Shared(id) => id.get() == Some(row.id),
        }
    }

    /// Selects `row`, or clears the selection when `row` is the selected one.
    fn toggle(self, row: FineRow) {
        match self {
            Self::Fine(current) => {
                let old = current.get_untracked();
                if let Some(old) = old {
                    old.selected.set(false);
                }
                if old.map(|old| old.id) == Some(row.id) {
                    current.set(None);
                } else {
                    row.selected.set(true);
                    current.set(Some(row));
                }
            }
            Self::Shared(id) => id.update(|id| {
                *id = if *id == Some(row.id) {
                    None
                } else {
                    Some(row.id)
                }
            }),
        }
    }
}

/// Returns the table with per-row signals. With `shared`, every row reads the selection signal.
pub fn view(count: usize, ticker: Option<Ticker>, shared: bool) -> impl IntoView {
    let mut ids = Ids::default();
    let rows = RwSignal::new(ids.rows_of::<FineRow>(count));
    let selection = if shared {
        Selection::Shared(RwSignal::new(None))
    } else {
        Selection::Fine(RwSignal::new(None))
    };
    let tick = RwSignal::new(0_u64);
    if let Some(ticker) = ticker {
        timer::every(
            Duration::from_millis(ticker.interval_ms),
            tick,
            move |now| {
                rows.with_untracked(|rows| {
                    for at in spec::ticker_positions(rows.len(), ticker.updates) {
                        let row = rows[at];
                        row.value.set(spec::ticker_value(row.id, now));
                    }
                });
            },
        );
    }
    let status = move || {
        let len = rows.with(Vec::len);
        match ticker {
            Some(_) => spec::ticker_status(len, tick.get()),
            None => spec::status(len),
        }
    };

    let list = dyn_stack(
        move || rows.get(),
        |row| row.id,
        move |row| table_row(row, selection),
    )
    .style(|s| s.flex_col().width_full());
    page::page(header(status, ()), list).on_event_stop(
        listener::KeyDown,
        move |_, event: &KeyboardEvent| {
            let Some(op) = keys::op_of(&event.key) else {
                return;
            };
            if op.changes_structure() {
                rows.update(|rows| op.apply(rows, &mut ids, count));
            } else {
                // The labels change in their own signals; the list stays as it is.
                rows.with_untracked(|rows| {
                    for mut row in rows.iter().step_by(10).copied() {
                        row.toggle_suffix();
                    }
                });
            }
        },
    )
}

/// Returns one row: a click selects it, or clears the selection when it is the selected one.
fn table_row(row: FineRow, selection: Selection) -> impl IntoView {
    band(
        Label::derived(move || row.label.get()),
        Label::derived(move || row.value.get()),
    )
    .style(move |s| selected_style(s, selection.is_selected(row)))
    .on_event_stop(listener::Click, move |_, _| selection.toggle(row))
}
