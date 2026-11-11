//! The search and typing scenarios: a header with a focused text input above `count` rows. With
//! `filter`, a keyed `for` shows the rows whose label holds the text; without, the rows stay.

use std::rc::Rc;

use zgui::prelude::*;
use zgui::reactive::{LocalStorage, RenderEffect};
use zgui_ui::prelude::*;
use zgui_ui_tokens::prelude::*;

use crate::page::{HeaderProps, PlainRowProps};
use crate::spec::{self, Ids, Row};

/// The page with the input and the rows.
#[component]
pub fn Search(count: usize, filter: bool) -> impl IntoView {
    let rows: Rc<Vec<Row>> = Rc::new(Ids::default().rows(count));
    let query: RwSignal<String, LocalStorage> = RwSignal::new_local(String::new());
    let field = NodeRef::new();
    let focus = RenderEffect::new(move |_| {
        field.get();
        field.focus();
    });
    on_cleanup_local(move || drop(focus));

    let shown = move || {
        if filter {
            query.with(|q| {
                rows.iter()
                    .filter(|r| spec::matches(&r.label, q))
                    .cloned()
                    .collect()
            })
        } else {
            rows.as_ref().clone()
        }
    };

    view! {
        column(class = "page") {
            Header(status = spec::status(count)) {
                box(class = "search-slot") {
                    ThemeProvider(scheme = ColorScheme::Light) {
                        Input(value = query, placeholder = "Search", node_ref = field)
                    }
                }
            }
            scroll(class = "list") {
                for row in move || shown(), key = |row: &Row| row.id {
                    PlainRow(row = row)
                }
            }
        }
    }
}
