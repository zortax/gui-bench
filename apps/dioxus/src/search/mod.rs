//! The search and typing scenarios: a header with the focused built-in text input above `count`
//! rows. With `filter`, the root reads the query and shows the rows whose label holds it, keyed
//! by id; without, only the input component reads the query and the rows stay.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::page::{self, Header, PlainRow};
use crate::spec::{self, Ids, Row};

/// The page with the input and the rows.
#[component]
pub fn Search(count: usize, filter: bool) -> Element {
    let rows: Rc<Vec<Row>> = use_hook(|| Rc::new(Ids::default().rows(count)));
    let query = use_signal(String::new);
    // Only the search scenario reads the query here, so typing renders the input alone.
    let shown: Vec<Row> = if filter {
        let query = query.read();
        rows.iter()
            .filter(|row| spec::matches(&row.label, &query))
            .cloned()
            .collect()
    } else {
        rows.as_ref().clone()
    };
    rsx! {
        style { {page::CSS} }
        div { class: "page",
            Header { status: spec::status(count),
                SearchField { query }
            }
            div { class: "list",
                for row in shown {
                    PlainRow { key: "{row.id}", row }
                }
            }
        }
    }
}

/// The built-in text input bound to `query`. It takes the focus once it is mounted, as the Blitz
/// TodoMVC example does, so typing goes into it without a click.
#[component]
fn SearchField(query: Signal<String>) -> Element {
    rsx! {
        input {
            class: "search",
            r#type: "text",
            placeholder: "Search",
            value: "{query}",
            oninput: move |event| query.set(event.value()),
            onmounted: move |event| async move {
                let _ = event.set_focus(true).await;
            },
        }
    }
}
