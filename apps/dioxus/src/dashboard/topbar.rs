//! The top bar: the page title, the search input, the export button and the avatar. Typing into
//! the search sets the query that filters the orders.

use dioxus::prelude::*;

use super::spec;

/// The white bar over the content.
#[component]
pub fn Topbar(query: Signal<String>) -> Element {
    rsx! {
        header { class: "topbar",
            span { class: "topbar__title", {spec::PAGE_TITLE} }
            SearchField { query }
            button { class: "button button--primary export", {spec::EXPORT_LABEL} }
            div { class: "topbar__avatar", {spec::USER_INITIALS} }
        }
    }
}

/// The built-in text input bound to `query`. Blitz does not paint the `placeholder` attribute, so
/// a span shows the placeholder while the query is empty; it lets the pointer through to the input.
#[component]
fn SearchField(query: Signal<String>) -> Element {
    rsx! {
        div { class: "search",
            input {
                class: "search__input",
                r#type: "text",
                placeholder: spec::SEARCH_PLACEHOLDER,
                value: "{query}",
                oninput: move |event| query.set(event.value()),
            }
            if query.read().is_empty() {
                span { class: "search__placeholder", {spec::SEARCH_PLACEHOLDER} }
            }
        }
    }
}
