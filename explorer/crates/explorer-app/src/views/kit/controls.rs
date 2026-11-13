//! Compact controls of the views: the view heading, the label of a size and the axis of sizes.

use explorer_core::units::{compact, thousands};
use leptos::prelude::*;

use crate::chart::Axis;
use crate::state::Size;

/// Returns the label of a size: the elements of the app, or the rows of the list.
pub fn size_label(size: Size) -> String {
    match size.1 {
        Some(e) => format!("{} elements", compact(f64::from(e))),
        None => format!("{} rows", compact(f64::from(size.0))),
    }
}

/// Returns the long label of a size, with the orders behind the elements.
pub fn size_words(size: Size) -> String {
    match size.1 {
        Some(e) => format!(
            "{} elements ({} orders)",
            thousands(f64::from(e)),
            thousands(f64::from(size.0))
        ),
        None => format!("{} rows", thousands(f64::from(size.0))),
    }
}

/// Returns the x axis over the sizes of a scenario: a log axis with a tick at every measured
/// size, named after what the sizes count.
pub fn size_axis(sizes: &[Size]) -> Axis {
    let label = if sizes.iter().any(|s| s.1.is_some()) {
        "elements in the app"
    } else {
        "rows of data"
    };
    let ticks = sizes
        .iter()
        .map(|s| f64::from(s.1.unwrap_or(s.0)))
        .collect();
    Axis::log_fixed(label, ticks)
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
