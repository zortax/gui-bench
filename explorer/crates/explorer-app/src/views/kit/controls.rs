//! Compact controls of the views: the view heading and a picker of sizes and refresh rates.

use leptos::prelude::*;

use crate::widgets::Segmented;

/// Returns the label of a size: the rows, with the ticker updates.
pub fn sizes_label(rows: u32, updates: Option<u32>) -> String {
    let r = explorer_core::units::thousands(f64::from(rows));
    match updates {
        Some(u) => format!("{r} × {u}/tick"),
        None => format!("{r} rows"),
    }
}

/// The heading of a view: a title, what it shows and controls on the right.
#[component]
pub fn ViewHead(
    #[prop(into)] title: String,
    #[prop(into)] text: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="gx-view__head">
            <h1>{title}</h1>
            <p>{text}</p>
            <div class="gx-view__controls">{children.map(|c| c())}</div>
        </div>
    }
}

/// A segmented picker of keys, such as sizes or refresh rates.
#[component]
pub fn SizePicker(
    #[prop(into)] options: Signal<Vec<(String, String)>>,
    value: RwSignal<String>,
    #[prop(optional, into)] title: Option<String>,
) -> impl IntoView {
    Effect::new(move |_| {
        let opts = options.get();
        if !opts.iter().any(|(k, _)| *k == value.get_untracked())
            && let Some((k, _)) = opts.last()
        {
            value.set(k.clone());
        }
    });
    view! {
        <Segmented
            options=options
            value=Signal::derive(move || value.get())
            on_change=Callback::new(move |k| value.set(k))
            title=title.unwrap_or_default()
        />
    }
}
