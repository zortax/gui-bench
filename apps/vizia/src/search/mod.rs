//! The search and typing scenarios: a header with the built-in `Textbox`, focused at startup,
//! above a `List` of `count` rows. With `filter`, the list shows the rows whose label holds the
//! text, through a memo of the text; without, it shows the rows of a signal that never changes.
//!
//! The textbox does not write its text back: `on_edit` sets the query signal on every edit. The
//! list compares its new rows with the old ones and rebuilds from the first that differs.

use vizia::prelude::*;

use crate::page;
use crate::spec::{self, Ids, Row};

/// Builds the page with the textbox and the rows.
pub fn build(cx: &mut Context, count: usize, filter: bool) {
    let rows: Vec<Row> = Ids::default().rows(count);
    let query = Signal::new(String::new());
    page::build(
        cx,
        spec::status(count),
        move |cx| {
            let field = Textbox::new(cx, query)
                .placeholder("Search")
                .on_edit(move |_, text| query.set(text))
                .class("search")
                .entity();
            cx.with_current(field, |cx| cx.focus());
        },
        move |cx| {
            if filter {
                let shown = Memo::new(move |_| {
                    query.with(|q| {
                        rows.iter()
                            .filter(|row| spec::matches(&row.label, q))
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                });
                list(cx, shown);
            } else {
                list(cx, Signal::new(rows));
            }
        },
    );
}

/// Builds the list of plain rows over `rows`.
fn list<V>(cx: &mut Context, rows: impl Res<V> + 'static)
where
    V: std::ops::Deref<Target = [Row]> + Clone + 'static,
{
    List::new(cx, rows, |cx, _, row| {
        page::plain_row(cx, &row.get());
    })
    .show_horizontal_scrollbar(false)
    .class("list");
}
