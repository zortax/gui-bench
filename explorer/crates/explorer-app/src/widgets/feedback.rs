//! Feedback while data loads or when it is missing: skeletons shaped like the final layout,
//! error cards with a retry, empty states with an action, badges and status dots.

use explorer_core::api::CheckStatus;
use leptos::prelude::*;

/// A skeleton of a view: a header line and a grid of chart cards.
#[component]
pub fn ViewSkeleton() -> impl IntoView {
    view! {
        <div class="gx-view gx-view--loading" aria-busy="true">
            <div class="gx-skel gx-skel--title"></div>
            <div class="gx-grid">
                {(0..4).map(|_| view! { <ChartSkeleton/> }).collect_view()}
            </div>
        </div>
    }
}

/// A skeleton of one chart card.
#[component]
pub fn ChartSkeleton(#[prop(optional)] height: Option<u32>) -> impl IntoView {
    let height = height.unwrap_or(260);
    view! {
        <div class="gx-card gx-card--skel" aria-busy="true">
            <div class="gx-skel gx-skel--line"></div>
            <div class="gx-skel gx-skel--note"></div>
            <div class="gx-skel gx-skel--plot" style:height=format!("{height}px")>
                <span></span><span></span><span></span><span></span>
            </div>
        </div>
    }
}

/// A card that reports an error, with an optional retry.
#[component]
pub fn ErrorCard(
    #[prop(into)] title: String,
    #[prop(into)] error: String,
    #[prop(optional)] retry: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="gx-card gx-card--error" role="alert">
            <div class="gx-error__title"><StatusDot status=CheckStatus::Fail/>{title}</div>
            <pre class="gx-error__text">{error}</pre>
            {retry.map(|r| view! {
                <button type="button" class="gx-btn" on:click=move |_| r.run(())>"Retry"</button>
            })}
        </div>
    }
}

/// An empty state that says why it is empty, with an optional action.
#[component]
pub fn Empty(
    #[prop(into)] message: String,
    #[prop(optional, into)] action: Option<String>,
    #[prop(optional)] on_action: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="gx-empty">
            <span>{message}</span>
            {action.zip(on_action).map(|(label, cb)| view! {
                <button type="button" class="gx-btn gx-btn--link" on:click=move |_| cb.run(())>{label}</button>
            })}
        </div>
    }
}

/// A small label in a tone.
#[component]
pub fn Badge(
    #[prop(into)] text: String,
    /// `ok`, `warn`, `err`, `info` or `muted`.
    #[prop(optional, into)]
    tone: Option<String>,
    #[prop(optional, into)] title: Option<String>,
) -> impl IntoView {
    view! { <span class="gx-badge" data-tone=tone.unwrap_or_else(|| "muted".into()) title=title>{text}</span> }
}

/// A dot in the tone of a check status.
#[component]
pub fn StatusDot(
    status: CheckStatus,
    #[prop(optional, into)] title: Option<String>,
) -> impl IntoView {
    view! { <span class="gx-dot" data-status=status.name() title=title></span> }
}
