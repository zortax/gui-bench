//! Compact input controls. Each is controlled: it shows a signal and reports changes through a
//! callback.

use leptos::prelude::*;

/// A row of mutually exclusive buttons.
#[component]
pub fn Segmented(
    /// The options as (key, label).
    #[prop(into)]
    options: Signal<Vec<(String, String)>>,
    #[prop(into)] value: Signal<String>,
    on_change: Callback<String>,
    /// A tooltip for the control.
    #[prop(optional, into)]
    title: Option<String>,
) -> impl IntoView {
    view! {
        <div class="gx-seg" role="radiogroup" title=title>
            <For
                each=move || options.try_get().unwrap_or_default()
                key=|(k, l)| (k.clone(), l.clone())
                children=move |(key, label)| {
                    let k = key.clone();
                    let active = Memo::new(move |_| value.try_get().is_some_and(|v| v == k));
                    view! {
                        <button
                            type="button"
                            class="gx-seg__opt"
                            role="radio"
                            aria-checked=move || active.get().to_string()
                            data-active=move || active.get().then_some("")
                            on:click=move |_| on_change.run(key.clone())
                        >
                            {label}
                        </button>
                    }
                }
            />
        </div>
    }
}

/// A labelled on/off switch.
#[component]
pub fn Toggle(
    #[prop(into)] label: String,
    #[prop(into)] value: Signal<bool>,
    on_change: Callback<bool>,
    #[prop(optional, into)] title: Option<String>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="gx-toggle"
            role="switch"
            title=title
            aria-checked=move || value.try_get().unwrap_or(false).to_string()
            data-on=move || value.try_get().unwrap_or(false).then_some("")
            on:click=move |_| on_change.run(!value.get_untracked())
        >
            <span class="gx-toggle__track"><span class="gx-toggle__knob"></span></span>
            <span>{label}</span>
        </button>
    }
}

/// A chip that toggles one item, with an optional color dot. Alt-click asks for the item alone.
#[component]
pub fn Chip(
    #[prop(into)] label: String,
    #[prop(into)] on: Signal<bool>,
    /// The color of the dot, a CSS color.
    #[prop(optional, into)]
    color: Option<String>,
    /// The color of the label text, a CSS color.
    #[prop(optional, into)]
    ink: Option<String>,
    on_toggle: Callback<()>,
    /// Called on alt-click or double-click: show this item alone.
    #[prop(optional)]
    on_solo: Option<Callback<()>>,
    #[prop(optional, into)] title: Option<String>,
) -> impl IntoView {
    let solo = on_solo;
    view! {
        <button
            type="button"
            class="gx-chip"
            title=title
            data-off=move || (!on.try_get().unwrap_or(true)).then_some("")
            style:--chip-ink=ink
            on:click=move |ev| {
                if ev.alt_key() && let Some(s) = solo {
                    s.run(());
                } else {
                    on_toggle.run(());
                }
            }
            on:dblclick=move |_| {
                if let Some(s) = solo {
                    s.run(());
                }
            }
        >
            {color.map(|c| view! { <span class="gx-chip__dot" style:background=c></span> })}
            <span class="gx-chip__label">{label}</span>
        </button>
    }
}

/// A compact native select.
#[component]
pub fn SelectBox(
    #[prop(into)] options: Signal<Vec<(String, String)>>,
    #[prop(into)] value: Signal<String>,
    on_change: Callback<String>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    view! {
        <label class="gx-select" title=title>
            {label.map(|l| view! { <span class="gx-select__label">{l}</span> })}
            <select
                class="gx-select__input"
                on:change=move |ev| on_change.run(event_target_value(&ev))
                prop:value=move || value.try_get().unwrap_or_default()
            >
                <For
                    each=move || options.try_get().unwrap_or_default()
                    key=|(k, l)| (k.clone(), l.clone())
                    children=move |(key, label)| {
                        let k = key.clone();
                        view! {
                            <option value=key selected=move || value.try_get().is_some_and(|v| v == k)>{label}</option>
                        }
                    }
                />
            </select>
        </label>
    }
}
