//! The sidebar: the brand, the nav items and the user block. A click on a nav item makes it the
//! active one; the stylesheet gives the other items their hover color.

use dioxus::prelude::*;

use super::spec;

/// The dark sidebar. `active` is the index of the active nav item.
#[component]
pub fn Sidebar(active: Signal<usize>) -> Element {
    rsx! {
        nav { class: "sidebar",
            div { class: "brand",
                div { class: "brand__logo" }
                span { class: "brand__name", {spec::APP_NAME} }
            }
            div { class: "nav",
                for (index , label) in spec::NAV_ITEMS.iter().enumerate() {
                    NavItem { key: "{index}", index, label, active }
                }
            }
            div { class: "user",
                div { class: "user__avatar", {spec::USER_INITIALS} }
                div {
                    div { class: "user__name", {spec::USER_NAME} }
                    div { class: "user__role", {spec::USER_ROLE} }
                }
            }
        }
    }
}

/// Nav item `index`: its icon, its label and, on the messages item, the unread badge.
#[component]
fn NavItem(index: usize, label: &'static str, active: Signal<usize>) -> Element {
    let is_active = active() == index;
    rsx! {
        button {
            class: "nav__item",
            class: if is_active { "active" },
            onclick: move |_| active.set(index),
            span { class: "nav__icon" }
            span { class: "nav__label", {label} }
            if index == spec::BADGE_ITEM {
                span { class: "badge", "{spec::BADGE_COUNT}" }
            }
        }
    }
}
