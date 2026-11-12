//! The size and refresh rate a view looks at, kept in the URL query (`size`, `rate`). A value
//! that the shown cells lack falls back to the largest size and the lowest rate, so the server
//! renders the same scope as the browser.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use super::controls::sizes_label;
use crate::state::{Selection, use_explorer, use_param};
use crate::widgets::Segmented;

/// A size: the rows, and the ticker updates per tick.
pub type Size = (u32, Option<u32>);

/// Returns the URL key of a size.
pub fn size_key(size: Size) -> String {
    format!("{}:{}", size.0, size.1.unwrap_or(0))
}

/// The scope of a view.
#[derive(Clone, Copy)]
pub struct Scope {
    pub sizes: Memo<Vec<Size>>,
    pub rates: Memo<Vec<f64>>,
    /// The picked size.
    pub size: Memo<Option<Size>>,
    /// The picked refresh rate.
    pub hz: Memo<Option<f64>>,
    set_size: Callback<String>,
    set_rate: Callback<String>,
}

impl Scope {
    /// Returns true when a cell key lies in the scope.
    pub fn holds(&self, rows: u32, updates: Option<u32>, hz: f64) -> bool {
        self.size.get() == Some((rows, updates)) && self.hz.get() == Some(hz)
    }
}

/// Returns the scope over the shown cells of a scenario.
pub fn use_scope(overview: Arc<RunOverview>, scenario: Signal<String>) -> Scope {
    let explorer = use_explorer();
    let (size_param, set_size) = use_param("size", "");
    let (rate_param, set_rate) = use_param("rate", "");
    let ov = overview.clone();
    let sizes = Memo::new(move |_| {
        let prefs = explorer.prefs.snapshot();
        Selection::new(&ov, &prefs, &scenario.get()).sizes()
    });
    let rates = Memo::new(move |_| {
        let prefs = explorer.prefs.snapshot();
        Selection::new(&overview, &prefs, &scenario.get()).rates()
    });
    let size = Memo::new(move |_| {
        let all = sizes.get();
        let want = size_param.get();
        all.iter()
            .copied()
            .find(|s| size_key(*s) == want)
            .or_else(|| all.last().copied())
    });
    let hz = Memo::new(move |_| {
        let all = rates.get();
        let want: Option<f64> = rate_param.get().parse().ok();
        all.iter()
            .copied()
            .find(|r| Some(*r) == want)
            .or_else(|| all.first().copied())
    });
    Scope {
        sizes,
        rates,
        size,
        hz,
        set_size,
        set_rate,
    }
}

/// The pickers of a scope.
#[component]
pub fn ScopeBar(
    scope: Scope,
    /// The word after a row count, such as `orders`.
    #[prop(optional)]
    noun: Option<&'static str>,
) -> impl IntoView {
    let sizes = Signal::derive(move || {
        scope
            .sizes
            .try_get()
            .unwrap_or_default()
            .into_iter()
            .map(|s| {
                let label = match noun {
                    Some(n) => format!("{} {n}", explorer_core::units::thousands(f64::from(s.0))),
                    None => sizes_label(s.0, s.1),
                };
                (size_key(s), label)
            })
            .collect::<Vec<_>>()
    });
    let rates = Signal::derive(move || {
        scope
            .rates
            .try_get()
            .unwrap_or_default()
            .into_iter()
            .map(|hz| (hz.to_string(), format!("{hz} Hz")))
            .collect::<Vec<_>>()
    });
    view! {
        <Segmented
            options=sizes
            value=Signal::derive(move || scope.size.try_get().flatten().map(size_key).unwrap_or_default())
            on_change=scope.set_size
            title="Size"
        />
        <Segmented
            options=rates
            value=Signal::derive(move || scope.hz.try_get().flatten().map(|h| h.to_string()).unwrap_or_default())
            on_change=scope.set_rate
            title="Refresh rate"
        />
    }
}
