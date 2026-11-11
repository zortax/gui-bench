//! A popover: a trigger button and a floating panel that closes on a click outside, on Escape
//! and on a click of a menu item.

use leptos::prelude::*;

/// A button that opens a floating panel below it.
#[component]
pub fn Popover(
    /// The content of the trigger button.
    button: ViewFn,
    /// The panel content.
    children: ChildrenFn,
    /// The panel aligns to the right edge of the trigger.
    #[prop(optional)]
    right: bool,
    #[prop(optional, into)] title: Option<String>,
    /// Extra classes of the trigger button.
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let children = StoredValue::new(children);
    view! {
        <div class="gx-pop" data-open=move || open.get().then_some("")>
            <button
                type="button"
                class=format!("gx-pop__trigger {}", class.unwrap_or_default())
                title=title
                aria-expanded=move || open.get().to_string()
                on:click=move |_| open.update(|o| *o = !*o)
            >
                {button.run()}
            </button>
            {move || open.try_get().unwrap_or(false).then(|| view! {
                <div class="gx-pop__scrim" on:click=move |_| open.set(false)></div>
                <div
                    class="gx-pop__panel"
                    data-right=right.then_some("")
                    tabindex="-1"
                    on:keydown=move |ev| {
                        if ev.key() == "Escape" {
                            open.set(false);
                        }
                    }
                    on:click=move |ev| {
                        let item = ev
                            .target()
                            .and_then(|t| wasm_bindgen::JsCast::dyn_into::<web_sys::Element>(t).ok())
                            .and_then(|e| e.closest(".gx-menu__item").ok().flatten());
                        if item.is_some() {
                            open.set(false);
                        }
                    }
                >
                    {children.read_value()()}
                </div>
            })}
        </div>
    }
}
