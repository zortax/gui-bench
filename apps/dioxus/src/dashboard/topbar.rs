//! The top bar: the title of the page, the search input, the live switch with its spinner, the
//! export button with its menu, and the avatar. Typing into the search sets the query that
//! filters the orders.

use dioxus::prelude::*;

use super::live::Live;
use super::spec;
use super::switch::Switch;

/// The white bar over the content. `active` is the nav item whose name is the title.
#[component]
pub fn Topbar(query: Signal<String>, active: ReadSignal<usize>, live: Live) -> Element {
    rsx! {
        header { class: "topbar",
            span { class: "topbar__title", {spec::NAV_ITEMS[active()]} }
            SearchField { query }
            div { class: "live",
                Switch { on: live.on, label: spec::LIVE_LABEL }
            }
            Spinner { on: live.on }
            ExportMenu {}
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

/// The square that turns while live is on, through the CSS keyframe animation `spin`.
#[component]
fn Spinner(on: ReadSignal<bool>) -> Element {
    rsx! {
        div { class: "spinner", class: if on() { "spinning" } }
    }
}

/// The export button and the menu it opens under itself. A click on the button or on an item
/// closes the menu.
#[component]
fn ExportMenu() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "export",
            button {
                class: "button button--primary export__button",
                aria_haspopup: "menu",
                aria_expanded: open(),
                onclick: move |_| open.toggle(),
                {spec::EXPORT_LABEL}
            }
            if open() {
                div { class: "menu", role: "menu",
                    for (index , label) in spec::MENU_ITEMS.into_iter().enumerate() {
                        button {
                            key: "{index}",
                            class: "menu__item",
                            role: "menuitem",
                            onclick: move |_| open.set(false),
                            {label}
                        }
                    }
                }
            }
        }
    }
}
