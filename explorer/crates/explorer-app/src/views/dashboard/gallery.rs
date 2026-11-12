//! The screenshots of the dashboard at one size and rate, one per framework and variant, so the
//! layouts compare side by side. A click opens the cell.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use crate::state::use_explorer;
use crate::views::kit::Scope;

/// The screenshot gallery.
#[component]
pub fn Gallery(overview: Arc<RunOverview>, scope: Scope) -> impl IntoView {
    let explorer = use_explorer();
    let run = overview.summary.id.clone();
    let shots = move || {
        let prefs = explorer.prefs.snapshot();
        crate::state::cells_of(&overview, &prefs, "dashboard")
            .into_iter()
            .filter(|c| scope.holds(c.key.rows, c.key.updates, c.key.hz))
            .map(|c| {
                let id = c.id;
                let src = format!("/files/{run}/{id}/screenshot.png");
                let label = c.key.label();
                view! {
                    <figure class="gx-shot" on:click=move |_| explorer.query.set("cell", Some(id.to_string()))>
                        <img src=src alt=format!("{label}: the dashboard") loading="lazy" width="1280" height="800"/>
                        <figcaption style:color=crate::palette::ink(&c.key.framework)>{label.clone()}</figcaption>
                    </figure>
                }
            })
            .collect_view()
    };
    view! { <div class="gx-shots">{shots}</div> }
}
