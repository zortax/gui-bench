//! The table scenario: a header with the row count above a keyed table of rows. The function
//! keys run the table operations of `spec::Op`, a click selects a row, and with a ticker a timer
//! updates rows and the header. Each variant holds the rows its own way.

mod coarse;
mod fine;
mod header;
mod keys;
mod ticker;

use crate::cli::{Scenario, Ticker, Variant};
use crate::page;

/// Opens the table window and runs the event loop until it closes.
pub fn run(scenario: Scenario, count: usize, variant: Variant, ticker: Option<Ticker>) {
    let title = scenario.title();
    match variant {
        Variant::Coarse => page::launch_app(title, coarse::CoarseTable { count, ticker }),
        v => page::launch_app(
            title,
            fine::FineTable {
                count,
                ticker,
                shared: v == Variant::Shared,
            },
        ),
    }
}
