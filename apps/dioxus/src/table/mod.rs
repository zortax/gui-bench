//! The table scenario: a header with the row count above a keyed table of rows. The function
//! keys run the table operations of `spec::Op`, a click selects a row, and with a ticker a timer
//! updates rows and the header. Each variant holds the rows its own way.

mod coarse;
mod fine;
mod header;
mod keys;
mod ticker;

use dioxus::prelude::*;

pub use coarse::CoarseTable;
pub use fine::FineTable;

/// Selects the row `id`, or clears the selection when `id` is the selected row.
fn toggle_selection(mut selected: Signal<Option<u64>>, id: u64) {
    selected.with_mut(|s| *s = if *s == Some(id) { None } else { Some(id) });
}
