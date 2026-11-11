//! The table scenario: a header with the row count above a table of rows. The function keys run
//! the table operations of `spec::Op`, a click selects a row, and with a ticker a timer updates
//! rows and the header. Each variant holds the rows its own way, in a model that handles the
//! events of the keys, the timer and the rows.

mod coarse;
mod fine;
mod keys;

use vizia::prelude::*;

use crate::cli::{Ticker, Variant};
use crate::spec::{self, Op};

/// The events that change the table, whatever its variant.
pub enum TableEvent {
    /// A function key ran an operation.
    Run(Op),
    /// The ticker timer fired.
    Tick,
}

/// Builds the table of `count` rows in `variant`, with the ticker when there is one.
pub fn build(cx: &mut Context, count: usize, variant: Variant, ticker: Option<Ticker>) {
    keys::listen(cx);
    if let Some(ticker) = ticker {
        start_ticker(cx, ticker);
    }
    match variant {
        Variant::Coarse => coarse::build(cx, count, ticker),
        v => fine::build(cx, count, ticker, v == Variant::Shared),
    }
}

/// Returns the header text: the row count, and the tick with a ticker.
fn status(len: usize, ticker: Option<Ticker>, tick: u64) -> String {
    match ticker {
        Some(_) => spec::ticker_status(len, tick),
        None => spec::status(len),
    }
}

/// Starts a timer that sends `TableEvent::Tick` every interval of the ticker.
fn start_ticker(cx: &mut Context, ticker: Ticker) {
    let interval = Duration::from_millis(ticker.interval_ms);
    let timer = cx.add_timer(interval, None, |cx, action| {
        if let TimerAction::Tick(_) = action {
            cx.emit(TableEvent::Tick);
        }
    });
    cx.start_timer(timer);
}
