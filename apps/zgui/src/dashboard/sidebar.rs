//! The sidebar: the brand, the nav items and the user at the bottom.

use zgui::prelude::*;
use zgui::reactive::Selector;

use super::spec;

/// The dark sidebar. A click on a nav item makes it the active one.
#[component]
pub fn NavSidebar() -> impl IntoView {
    let active = RwSignal::new(0_usize);
    let selector = Selector::new(move || active.get());
    view! {
        column(class = "sidebar") {
            row(class = "brand") {
                box(class = "logo")
                text(class = "brand__name") {{spec::APP_NAME}}
            }
            column(class = "nav") {
                {(0..spec::NAV_ITEMS.len())
                    .map(|index| view! {
                        NavItem(index = index, active = active, selector = selector.clone())
                    }.into_view())
                    .collect::<Vec<_>>()}
            }
            row(class = "user") {
                row(class = "user__avatar") {{spec::USER_INITIALS}}
                column(class = "user__text") {
                    text(class = "user__name") {{spec::USER_NAME}}
                    text(class = "user__role") {{spec::USER_ROLE}}
                }
            }
        }
    }
}

/// Nav item `index`: its icon, its label and, on one item, the unread badge.
#[component]
fn NavItem(index: usize, active: RwSignal<usize>, selector: Selector<usize>) -> impl IntoView {
    let badge = (index == spec::BADGE_ITEM).then(|| {
        view! {
            row(class = "badge") {{spec::BADGE_COUNT.to_string()}}
        }
    });
    view! {
        control(
            class = "nav__item",
            class:active = move || selector.is_selected(&index),
            on:click = move |_| active.set(index)
        ) {
            box(class = "nav__icon")
            text {{spec::NAV_ITEMS[index]}}
            {badge}
        }
    }
}
