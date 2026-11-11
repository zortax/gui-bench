//! The table header as its own component: it reads the row list and the tick, so a tick renders
//! the header alone and not the table around it.

use dioxus::prelude::*;

use crate::page::Header;
use crate::spec;

/// The header of a table whose list holds `T`s. With `ticker`, it shows the tick too.
#[component]
pub fn TableHeader<T: Clone + PartialEq + 'static>(
    rows: Signal<Vec<T>>,
    tick: Signal<u64>,
    ticker: bool,
) -> Element {
    let len = rows.read().len();
    let status = if ticker {
        spec::ticker_status(len, tick())
    } else {
        spec::status(len)
    };
    rsx! {
        Header { status }
    }
}
