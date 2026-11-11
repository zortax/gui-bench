//! The fine and shared variants: a keyed list of row components that each get their label and
//! value as states of their own. An edit writes the states of the rows it changes, so only those
//! row components render again; a structural operation changes the list state, and the keyed
//! diff keeps every row component whose id and props stay.
//!
//! Selection: in the fine variant the table reads the selection and hands each row an
//! `is_selected` prop, so a selection change renders the table, and of the rows only the two
//! whose prop changed (freya skips components whose props are equal). In the shared variant
//! every row reads the one selection state itself, so a selection change renders every row.

use freya::prelude::*;

use super::header::TableHeader;
use super::keys;
use super::ticker::use_ticker;
use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids, Row, TableRow};

/// A row with its label and value in states.
#[derive(Clone, Copy, PartialEq)]
struct FineRow {
    id: u64,
    label: State<String>,
    value: State<u32>,
}

impl TableRow for FineRow {
    fn from_row(row: Row) -> Self {
        Self {
            id: row.id,
            label: State::create(row.label),
            value: State::create(row.value),
        }
    }

    fn toggle_suffix(&mut self) {
        let mut label = self.label.write();
        *label = spec::toggle_suffix(&label);
    }
}

/// The table with per-row states. With `shared`, every row reads the selection state.
pub struct FineTable {
    pub count: usize,
    pub ticker: Option<Ticker>,
    pub shared: bool,
}

impl App for FineTable {
    fn render(&self) -> impl IntoElement {
        let count = self.count;
        let shared = self.shared;
        let mut ids = use_state(Ids::default);
        let mut rows = use_state(|| ids.write().rows_of::<FineRow>(count));
        let selected = use_state(|| None::<u64>);
        let tick = use_ticker(self.ticker, move |now, updates| {
            let rows = rows.peek();
            for at in spec::ticker_positions(rows.len(), updates) {
                let mut value = rows[at].value;
                value.set(spec::ticker_value(rows[at].id, now));
            }
        });

        let on_key_down = move |e: Event<KeyboardEventData>| {
            let Some(op) = keys::op_of(&e.key) else {
                return;
            };
            if op.changes_structure() {
                op.apply(&mut rows.write(), &mut ids.write(), count);
            } else {
                // The label edit writes the row states only and leaves the list as it is.
                for mut row in rows.peek().iter().step_by(10).copied() {
                    row.toggle_suffix();
                }
            }
        };

        // Only the fine variant reads the selection here; shared rows read it themselves.
        let current = if shared { None } else { *selected.read() };
        let children: Vec<Element> = rows
            .read()
            .iter()
            .map(|&row| {
                FineRowView {
                    row,
                    selected,
                    is_selected: (!shared).then_some(current == Some(row.id)),
                    key: DiffKey::None,
                }
                .key(row.id)
                .into_element()
            })
            .collect();

        page::page(
            TableHeader {
                rows,
                tick,
                ticker: self.ticker.is_some(),
            },
            children,
        )
        .on_global_key_down(on_key_down)
    }
}

/// One row: a click selects it, or clears the selection when it is the selected one. With
/// `is_selected` the parent says whether it is selected; without, the row reads `selected`.
#[derive(PartialEq)]
struct FineRowView {
    row: FineRow,
    selected: State<Option<u64>>,
    is_selected: Option<bool>,
    key: DiffKey,
}

impl KeyExt for FineRowView {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for FineRowView {
    fn render(&self) -> impl IntoElement {
        let id = self.row.id;
        let mut selected = self.selected;
        let is_selected = self
            .is_selected
            .unwrap_or_else(|| *selected.read() == Some(id));
        page::row(
            self.row.label.read().clone(),
            *self.row.value.read(),
            is_selected,
        )
        .on_press(move |_| {
            let mut selected = selected.write();
            *selected = if *selected == Some(id) {
                None
            } else {
                Some(id)
            };
        })
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
