//! The coarse variant: one signal holds every row as plain data and one more the selection, and
//! the rows are built in a `Binding` on each. Any change, an edit of one row or a new selection
//! included, runs a binding again, which removes every row and builds them all anew.

use vizia::prelude::*;

use super::TableEvent;
use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids, Row};

/// The events of the rows.
enum CoarseEvent {
    /// The row with this id was clicked.
    Select(u64),
}

/// The state of the table.
struct CoarseTable {
    ids: Ids,
    count: usize,
    updates: usize,
    rows: Signal<Vec<Row>>,
    tick: Signal<u64>,
    selected: Signal<Option<u64>>,
}

impl Model for CoarseTable {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|table_event, _| match *table_event {
            TableEvent::Run(op) => {
                self.rows
                    .update(|rows| op.apply(rows, &mut self.ids, self.count));
            }
            TableEvent::Tick => {
                let now = self.tick.get() + 1;
                self.tick.set(now);
                self.rows.update(|rows| {
                    for at in spec::ticker_positions(rows.len(), self.updates) {
                        rows[at].value = spec::ticker_value(rows[at].id, now);
                    }
                });
            }
        });
        event.map(|CoarseEvent::Select(id), _| {
            let id = *id;
            self.selected
                .update(|s| *s = if *s == Some(id) { None } else { Some(id) });
        });
    }
}

/// Builds the table with one signal for all rows.
pub fn build(cx: &mut Context, count: usize, ticker: Option<Ticker>) {
    let mut ids = Ids::default();
    let rows = Signal::new(ids.rows(count));
    let tick = Signal::new(0_u64);
    let selected = Signal::new(None::<u64>);
    CoarseTable {
        ids,
        count,
        updates: ticker.map_or(0, |ticker| ticker.updates),
        rows,
        tick,
        selected,
    }
    .build(cx);

    let status = Memo::new(move |_| super::status(rows.with(Vec::len), ticker, tick.get()));
    page::build(
        cx,
        status,
        |_| {},
        move |cx| {
            page::scroll(cx, move |cx| {
                Binding::new(cx, selected, move |cx| {
                    Binding::new(cx, rows, move |cx| {
                        let selected = selected.get();
                        rows.with(|rows| {
                            for row in rows {
                                table_row(cx, row, selected == Some(row.id));
                            }
                        });
                    });
                });
            });
        },
    );
}

/// Builds one row with its label and value as plain text. A click selects the row.
fn table_row(cx: &mut Context, row: &Row, is_selected: bool) {
    let (id, label, value) = (row.id, row.label.clone(), row.value);
    HStack::new(cx, move |cx| {
        Label::new(cx, label).class("label").hoverable(false);
        Label::new(cx, value).class("value").hoverable(false);
    })
    .class("row")
    .toggle_class("selected", is_selected)
    .on_press(move |cx| cx.emit(CoarseEvent::Select(id)));
}
