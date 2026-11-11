//! The table header as its own component: it reads the row list and the tick, so a tick renders
//! the header alone and not the table around it.

use freya::prelude::*;

use crate::page;
use crate::spec;

/// The header of a table whose rows are `T`. With `ticker`, it shows the tick too.
#[derive(PartialEq)]
pub struct TableHeader<T: 'static> {
    pub rows: State<Vec<T>>,
    pub tick: State<u64>,
    pub ticker: bool,
}

impl<T: PartialEq + 'static> Component for TableHeader<T> {
    fn render(&self) -> impl IntoElement {
        let len = self.rows.read().len();
        let status = if self.ticker {
            spec::ticker_status(len, *self.tick.read())
        } else {
            spec::status(len)
        };
        page::header(status, rect())
    }
}
