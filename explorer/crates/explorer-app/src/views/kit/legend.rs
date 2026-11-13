//! The legend of a scenario: its shown frameworks, with editions dashed, and the encodings that
//! every chart of the story shares; and the empty state of a scenario without shown cells.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use crate::chart::{Legend, LegendItem};
use crate::palette::{self, Marker};
use crate::state::{Selection, use_explorer};
use crate::widgets::Empty;

/// The legend bar of a scenario.
#[component]
pub fn StoryLegend(
    overview: Arc<RunOverview>,
    scenario: &'static str,
    /// Add the entry of the sizes that did not run.
    #[prop(optional)]
    stops: bool,
) -> impl IntoView {
    let explorer = use_explorer();
    let items = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let mut items: Vec<LegendItem> = Selection::new(&overview, &prefs, scenario)
            .frameworks()
            .iter()
            .map(|f| {
                let mut item = LegendItem::framework(f);
                if palette::is_edition(f) {
                    item.dash = Some(palette::dash(f).to_owned());
                }
                item
            })
            .collect();
        items.push(LegendItem::encoding(
            "built by hand in the app",
            Marker::Circle,
            None,
            true,
        ));
        if stops {
            items.push(LegendItem::encoding(
                "not run: over 250 ms before",
                Marker::Cross,
                None,
                false,
            ));
        }
        items
    });
    view! {
        <div class="gx-legendbar">
            <Legend items=items/>
            <span class="gx-legendbar__note">"ring: the host disturbed the measurement · dashed: an edition of a framework"</span>
        </div>
    }
}

/// The empty state of a scenario without shown cells.
#[component]
pub fn NoCells(scenario: &'static str) -> impl IntoView {
    let explorer = use_explorer();
    view! {
        <Empty
            message=format!("No {scenario} cells shown: the run has none, or the filters hide them.")
            action="Show every framework and rate"
            on_action=Callback::new(move |_| explorer.query.set_all(vec![("hide", None), ("hz", None)]))
        />
    }
}
