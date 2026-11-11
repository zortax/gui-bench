//! The coarse variant: one state holds every row, and the root component builds the rows inline
//! from it and the selection. Any change, an edit of one row or a new selection included, renders
//! the root again and builds every row anew.

use freya::prelude::*;

use super::header::TableHeader;
use super::keys;
use super::ticker::use_ticker;
use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids, Row};

/// The table with one state for all rows.
pub struct CoarseTable {
    pub count: usize,
    pub ticker: Option<Ticker>,
}

impl App for CoarseTable {
    fn render(&self) -> impl IntoElement {
        let count = self.count;
        let mut ids = use_state(Ids::default);
        let mut rows = use_state(|| ids.write().rows(count));
        let mut selected = use_state(|| None::<u64>);
        let tick = use_ticker(self.ticker, move |now, updates| {
            let mut rows = rows.write();
            for at in spec::ticker_positions(rows.len(), updates) {
                rows[at].value = spec::ticker_value(rows[at].id, now);
            }
        });

        let on_key_down = move |e: Event<KeyboardEventData>| {
            if let Some(op) = keys::op_of(&e.key) {
                op.apply(&mut rows.write(), &mut ids.write(), count);
            }
        };

        let current = *selected.read();
        let children: Vec<Element> = rows
            .read()
            .iter()
            .map(|row: &Row| {
                let id = row.id;
                page::row(row.label.clone(), row.value, current == Some(id))
                    .key(id)
                    .on_press(move |_| {
                        let mut selected = selected.write();
                        *selected = if *selected == Some(id) {
                            None
                        } else {
                            Some(id)
                        };
                    })
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
