//! The ticker timer: a task of the table component that waits the interval with freya's timer,
//! counts the tick up and lets the variant update its rows.

use std::time::Duration;

use freya::prelude::*;

use crate::cli::Ticker;

/// Returns the tick state. With a ticker, starts a task that every interval counts the tick up
/// and calls `update` with the new tick and the number of rows to update. The task lives as
/// long as the component.
pub fn use_ticker(
    ticker: Option<Ticker>,
    mut update: impl FnMut(u64, usize) + 'static,
) -> State<u64> {
    let mut tick = use_state(|| 0_u64);
    use_hook(move || {
        let Some(ticker) = ticker else { return };
        spawn(async move {
            loop {
                timer(Duration::from_millis(ticker.interval_ms)).await;
                let now = *tick.peek() + 1;
                tick.set(now);
                update(now, ticker.updates);
            }
        });
    });
    tick
}
