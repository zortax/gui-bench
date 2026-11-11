//! The ticker timer: a future of the table component that waits the interval with tokio's sleep,
//! as the Dioxus docs of `use_future` do, counts the tick up and lets the variant update its rows.

use std::time::Duration;

use dioxus::prelude::*;

use crate::cli::Ticker;

/// Returns the tick signal. With a ticker, starts a future that every interval counts the tick up
/// and calls `update` with the new tick and the number of rows to update. The future lives as
/// long as the component.
pub fn use_ticker(
    ticker: Option<Ticker>,
    update: impl FnMut(u64, usize) + Copy + 'static,
) -> Signal<u64> {
    let mut tick = use_signal(|| 0_u64);
    use_future(move || async move {
        let Some(ticker) = ticker else { return };
        let mut update = update;
        loop {
            tokio::time::sleep(Duration::from_millis(ticker.interval_ms)).await;
            let now = *tick.peek() + 1;
            tick.set(now);
            update(now, ticker.updates);
        }
    });
    tick
}
