//! The coarse variant: one signal holds every row, and the root component builds the rows inline
//! from it and the selection. Any change, an edit of one row or a new selection included, renders
//! the root again and builds and diffs every row anew.

use dioxus::prelude::*;

use super::header::TableHeader;
use super::keys::use_function_keys;
use super::ticker::use_ticker;
use super::toggle_selection;
use crate::cli::Ticker;
use crate::page;
use crate::spec::{self, Ids};

/// The table with one signal for all rows.
#[component]
pub fn CoarseTable(count: usize, ticker: Option<Ticker>) -> Element {
    let mut ids = use_signal(Ids::default);
    let mut rows = use_signal(|| ids.write().rows(count));
    let selected = use_signal(|| None::<u64>);
    let tick = use_ticker(ticker, move |now, updates| {
        let mut rows = rows.write();
        for at in spec::ticker_positions(rows.len(), updates) {
            rows[at].value = spec::ticker_value(rows[at].id, now);
        }
    });
    use_function_keys(move |op| op.apply(&mut rows.write(), &mut ids.write(), count));

    let current = selected();
    let list = rows.read();
    let children = list.iter().map(|row| {
        let id = row.id;
        rsx! {
            div {
                key: "{id}",
                class: "row",
                class: if current == Some(id) { "selected" },
                onclick: move |_| toggle_selection(selected, id),
                span { class: "row__label", "{row.label}" }
                span { class: "row__value", "{row.value}" }
            }
        }
    });
    rsx! {
        style { {page::CSS} }
        div { class: "page",
            TableHeader { rows, tick, ticker: ticker.is_some() }
            div { class: "list", {children} }
        }
    }
}
