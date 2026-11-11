//! The spinner scenario: a header whose spinner turns once a second through a CSS keyframe
//! animation, above static rows.

use zgui::prelude::*;

use crate::page::{HeaderProps, PlainRowProps};
use crate::spec::{self, Ids};

/// The page with `count` static rows and the spinner.
#[component]
pub fn Spinner(count: usize) -> impl IntoView {
    let rows = Ids::default().rows(count);
    view! {
        column(class = "page") {
            Header(status = spec::status(count)) {
                box(class = "spinner") {}
            }
            scroll(class = "list") {
                {rows.into_iter().map(|row| AnyView::new(view! { PlainRow(row = row) })).collect::<Vec<_>>()}
            }
        }
    }
}
