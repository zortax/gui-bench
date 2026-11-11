//! The theme switch: auto (the system setting), light or dark. The choice sits in local storage
//! and in the `data-theme` attribute of the document.

use leptos::prelude::*;

use crate::widgets::{Icon, IconKind};

const KEY: &str = "gx-theme";

fn stored() -> String {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(KEY).ok().flatten())
        .unwrap_or_default()
}

fn apply(theme: &str) {
    let Some(root) = document().document_element() else {
        return;
    };
    if let Ok(Some(storage)) = window().local_storage() {
        let _ = if theme.is_empty() {
            storage.remove_item(KEY)
        } else {
            storage.set_item(KEY, theme)
        };
    }
    if theme.is_empty() {
        let _ = root.remove_attribute("data-theme");
    } else {
        let _ = root.set_attribute("data-theme", theme);
    }
}

/// A button that cycles auto, light and dark.
#[component]
pub fn ThemeSwitch() -> impl IntoView {
    let theme = RwSignal::new(String::new());
    Effect::new(move |_| theme.set(stored()));
    let next = move |_| {
        let t = match theme.get_untracked().as_str() {
            "" => "light",
            "light" => "dark",
            _ => "",
        };
        apply(t);
        theme.set(t.to_owned());
    };
    view! {
        <button
            type="button"
            class="gx-iconbtn"
            on:click=next
            title=move || match theme.get().as_str() {
                "light" => "Theme: light (click for dark)",
                "dark" => "Theme: dark (click for system)",
                _ => "Theme: system (click for light)",
            }
        >
            {move || match theme.get().as_str() {
                "light" => view! { <Icon kind=IconKind::Sun/> },
                "dark" => view! { <Icon kind=IconKind::Moon/> },
                _ => view! { <Icon kind=IconKind::Monitor/> },
            }}
        </button>
    }
}
