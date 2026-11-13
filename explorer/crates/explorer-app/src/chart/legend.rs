//! The legend row of a chart: a swatch and the name of each framework or encoding. A click hides
//! a framework in every chart, alt-click shows it alone, and hovering highlights it.

use leptos::prelude::*;

use super::card::use_card;
use crate::palette::Marker;
use crate::state::use_explorer;

/// One legend entry.
#[derive(Debug, Clone, PartialEq)]
pub struct LegendItem {
    /// The framework the entry hides and highlights; empty for an encoding entry.
    pub group: String,
    pub label: String,
    pub color: String,
    pub ink: String,
    pub marker: Marker,
    /// The dash of a line swatch; `None` draws the marker alone.
    pub dash: Option<String>,
    pub hollow: bool,
}

impl LegendItem {
    /// Returns the entry of a framework.
    pub fn framework(framework: &str) -> Self {
        Self {
            group: framework.to_owned(),
            label: framework.to_owned(),
            color: crate::palette::mark(framework),
            ink: crate::palette::ink(framework),
            marker: Marker::Circle,
            dash: None,
            hollow: false,
        }
    }

    /// Returns an encoding entry in muted ink, such as a part built by hand.
    pub fn encoding(
        label: impl Into<String>,
        marker: Marker,
        dash: Option<String>,
        hollow: bool,
    ) -> Self {
        Self {
            group: String::new(),
            label: label.into(),
            color: "var(--muted)".to_owned(),
            ink: "var(--text-2)".to_owned(),
            marker,
            dash,
            hollow,
        }
    }
}

/// A legend row.
#[component]
pub fn Legend(#[prop(into)] items: Signal<Vec<LegendItem>>) -> impl IntoView {
    let ctx = use_card();
    let explorer = use_explorer();
    let all = Memo::new(move |_| {
        items
            .get()
            .iter()
            .filter(|i| !i.group.is_empty())
            .map(|i| i.group.clone())
            .collect::<Vec<_>>()
    });
    view! {
        <div class="gx-legend">
            <For
                each=move || items.get()
                key=|i| (i.group.clone(), i.label.clone())
                children=move |item| {
                    let group = item.group.clone();
                    let g2 = group.clone();
                    let g3 = group.clone();
                    let swatch = match &item.dash {
                        Some(dash) => view! {
                            <svg class="gx-legend__swatch" viewBox="0 0 22 10" width="22" height="10">
                                <line x1="1" x2="21" y1="5" y2="5" style:stroke=item.color.clone() stroke-dasharray=dash.clone()/>
                                <path d=item.marker.path(3.0) transform="translate(11 5)" style:fill=if item.hollow { "var(--surface)".to_owned() } else { item.color.clone() } style:stroke=item.color.clone()/>
                            </svg>
                        }.into_any(),
                        None => view! {
                            <svg class="gx-legend__swatch" viewBox="0 0 10 10" width="10" height="10">
                                <path d=item.marker.path(3.6) transform="translate(5 5)" style:fill=if item.hollow { "var(--surface)".to_owned() } else { item.color.clone() } style:stroke=item.color.clone()/>
                            </svg>
                        }.into_any(),
                    };
                    view! {
                        <button
                            type="button"
                            class="gx-legend__item"
                            data-static=group.is_empty().then_some("")
                            title=(!group.is_empty()).then_some("click: hide · alt+click: show alone")
                            style:color=item.ink.clone()
                            on:mouseenter=move |_| if !g2.is_empty() { ctx.highlight.set(Some(g2.clone())) }
                            on:mouseleave=move |_| ctx.highlight.set(None)
                            on:click=move |ev| {
                                if g3.is_empty() {
                                    return;
                                }
                                let mut hidden = explorer.prefs.hidden.get_untracked();
                                if ev.alt_key() {
                                    let mut rest: Vec<String> = explorer.prefs.hidden.get_untracked();
                                    rest.retain(|h| *h != g3);
                                    let others: Vec<String> = all.get_untracked().into_iter().filter(|f| *f != g3).collect();
                                    for o in others {
                                        if !rest.contains(&o) {
                                            rest.push(o);
                                        }
                                    }
                                    hidden = rest;
                                } else if !hidden.contains(&g3) {
                                    hidden.push(g3.clone());
                                }
                                explorer.query.set("hide", (!hidden.is_empty()).then(|| hidden.join(",")));
                            }
                        >
                            {swatch}
                            <span>{item.label.clone()}</span>
                        </button>
                    }
                }
            />
        </div>
    }
}
