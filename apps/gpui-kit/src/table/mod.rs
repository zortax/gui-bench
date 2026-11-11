//! The table scenario: a header with the row count above a keyed table of rows. The function
//! keys run the table operations of `spec::Op`, a click selects a row, and with a ticker a timer
//! updates rows and the header. Each variant holds the rows its own way.

mod coarse;
mod fine;
pub mod keys;
mod ticker;

pub use self::coarse::CoarseTable;
pub use self::fine::FineTable;
use crate::cli::Ticker;
use crate::spec;

/// Returns the header status of a table with `len` rows at `tick`.
fn status(len: usize, ticker: Option<Ticker>, tick: u64) -> String {
    match ticker {
        Some(_) => spec::ticker_status(len, tick),
        None => spec::status(len),
    }
}

/// Returns the selection after a click on row `id`: the row, or none when it was selected.
fn toggle(selected: Option<u64>, id: u64) -> Option<u64> {
    if selected == Some(id) { None } else { Some(id) }
}
