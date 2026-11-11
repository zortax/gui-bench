//! The search and typing scenarios: a header with floem's text input, focused at startup, above
//! `count` rows. With `filter`, a keyed `dyn_stack` shows the rows whose label holds the text;
//! without, the rows are one static stack.
//!
//! floem resolves a focus request against the styled view tree, which does not exist yet while
//! the views are built, so a request made then finds nothing focusable and is dropped. The input
//! therefore asks for focus again when a zero timer fires, which runs in the event loop after the
//! first frame is styled (floem's dropdown defers its focus request the same way).

use std::rc::Rc;
use std::time::Duration;

use floem::action::exec_after;
use floem::prelude::*;
use floem::reactive::Trigger;

use crate::page::{self, header, plain_row, plain_rows};
use crate::spec::{self, Ids, Row};

/// Returns the page with the input and the rows.
pub fn view(count: usize, filter: bool) -> impl IntoView {
    let rows = Ids::default().rows(count);
    let query = RwSignal::new(String::new());
    let styled = Trigger::new();
    exec_after(Duration::ZERO, move |_| styled.notify());
    let input = TextInput::new(query)
        .placeholder("Search")
        .style(|s| {
            s.flex_shrink(0.0_f32)
                .width(f64::from(spec::INPUT_WIDTH))
                .height(f64::from(spec::INPUT_HEIGHT))
        })
        .request_focus(move || styled.track());
    let header = header(move || spec::status(count), input);
    if filter {
        let rows = Rc::new(rows);
        let shown = dyn_stack(
            move || {
                query.with(|q| {
                    rows.iter()
                        .filter(|row| spec::matches(&row.label, q))
                        .cloned()
                        .collect::<Vec<Row>>()
                })
            },
            |row| row.id,
            plain_row,
        )
        .style(|s| s.flex_col().width_full());
        page::page(header, shown)
    } else {
        page::page(header, plain_rows(rows))
    }
}
