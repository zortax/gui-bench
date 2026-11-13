//! The screenshots of the app at one size and rate, one per framework, so the layouts compare
//! side by side. A click opens the cell.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use crate::state::use_explorer;
use crate::views::kit::{APP, Scope};

/// The screenshot gallery.
#[component]
pub fn Gallery(overview: Arc<RunOverview>, scope: Scope) -> impl IntoView {
    let explorer = use_explorer();
    let run = overview.summary.id.clone();
    let shots = move || {
        let prefs = explorer.prefs.snapshot();
        crate::state::cells_of(&overview, &prefs, APP)
            .into_iter()
            .filter(|c| scope.holds(c.key.rows, c.key.hz))
            .map(|c| {
                let id = c.id;
                let src = format!("/files/{run}/{id}/screenshot.png");
                let label = c.key.framework.clone();
                view! {
                    <figure class="gx-shot" on:click=move |_| explorer.query.set("cell", Some(id.to_string()))>
                        <img src=src alt=format!("{label}: the app after startup") loading="lazy" width="1280" height="800"/>
                        <figcaption style:color=crate::palette::ink(&c.key.framework)>{label.clone()}</figcaption>
                    </figure>
                }
            })
            .collect_view()
    };
    view! { <div class="gx-shots">{shots}</div> }
}
