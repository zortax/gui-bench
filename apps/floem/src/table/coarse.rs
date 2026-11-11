//! The coarse variant: one signal holds every row, and the list is one `dyn_container` over it
//! and the selection. Any change, an edit of one row or a new selection included, runs the
//! container again, which drops the old subtree and builds every row anew.

use std::time::Duration;

use floem::prelude::*;

use super::{keys, timer};
use crate::cli::Ticker;
use crate::page::{self, band, header, selected_style};
use crate::spec::{self, Ids, Row};

/// Returns the table with one signal for all rows.
pub fn view(count: usize, ticker: Option<Ticker>) -> impl IntoView {
    let mut ids = Ids::default();
    let rows = RwSignal::new(ids.rows(count));
    let selected = RwSignal::new(None::<u64>);
    let tick = RwSignal::new(0_u64);
    if let Some(ticker) = ticker {
        timer::every(
            Duration::from_millis(ticker.interval_ms),
            tick,
            move |now| {
                rows.update(|rows| {
                    for at in spec::ticker_positions(rows.len(), ticker.updates) {
                        rows[at].value = spec::ticker_value(rows[at].id, now);
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

    let list = dyn_container(
        move || (rows.get(), selected.get()),
        move |(rows, current)| {
            Stack::vertical_from_iter(
                rows.into_iter()
                    .map(|row| table_row(row, current, selected)),
            )
            .style(|s| s.width_full())
        },
    )
    .style(|s| s.width_full());
    page::page(header(status, ()), list).on_event_stop(
        listener::KeyDown,
        move |_, event: &KeyboardEvent| {
            if let Some(op) = keys::op_of(&event.key) {
                rows.update(|rows| op.apply(rows, &mut ids, count));
            }
        },
    )
}

/// Builds one row: a click selects it, or clears the selection when it is the selected one.
fn table_row(row: Row, current: Option<u64>, selected: RwSignal<Option<u64>>) -> impl IntoView {
    let id = row.id;
    band(Label::new(row.label), Label::new(row.value))
        .style(move |s| selected_style(s, current == Some(id)))
        .on_event_stop(listener::Click, move |_, _| {
            selected.update(|s| *s = if *s == Some(id) { None } else { Some(id) });
        })
}
