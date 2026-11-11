//! The search and typing scenarios: a header with the focused built-in text input above `count`
//! rows. With `filter`, the root reads the input state and shows the rows whose label holds the
//! text, keyed by id; without, only the input reads its state and the rows stay.

use std::rc::Rc;

use freya::prelude::*;

use crate::page::{self, PlainRow};
use crate::spec::{self, Ids, Row};

/// Opens the search window and runs the event loop until it closes.
pub fn run(title: &'static str, count: usize, filter: bool) {
    page::launch_app(title, SearchPage { count, filter });
}

/// The page with the input and the rows.
struct SearchPage {
    count: usize,
    filter: bool,
}

impl App for SearchPage {
    fn render(&self) -> impl IntoElement {
        let count = self.count;
        let rows: Rc<Vec<Row>> = use_hook(|| Rc::new(Ids::default().rows(count)));
        let query = use_state(String::new);

        let shown: Vec<Element> = if self.filter {
            let query = query.read();
            rows.iter()
                .filter(|row| spec::matches(&row.label, &query))
                .map(|row| PlainRow::new(row.clone()).key(row.id).into_element())
                .collect()
        } else {
            rows.iter()
                .map(|row| PlainRow::new(row.clone()).into_element())
                .collect()
        };

        let input = Input::new(query)
            .placeholder("Search")
            .compact()
            .auto_focus(true)
            .width(Size::px(spec::INPUT_WIDTH));
        page::page(page::header(spec::status(count), input), shown)
    }
}
