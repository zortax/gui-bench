//! The size and refresh rate a view looks at, kept in the URL query (`size`, `rate`). A rate
//! that the shown cells lack falls back to the rate of every cell; a size they lack falls back
//! to the default of the view, so the server renders the same scope as the browser.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use leptos::prelude::*;

use super::controls::size_label;
use crate::state::{Selection, Size, use_explorer, use_param};
use crate::widgets::Segmented;

/// The size a scope takes without a choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeDefault {
    /// The smallest size.
    Smallest,
    /// The size that the most frameworks ran, the larger one on a tie.
    MostCells,
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
    pub fn holds(&self, rows: u32, hz: f64) -> bool {
        self.size.get().map(|s| s.0) == Some(rows) && self.hz.get() == Some(hz)
    }
}

/// Returns the scope over the shown cells of a scenario.
pub fn use_scope(
    overview: Arc<RunOverview>,
    scenario: Signal<String>,
    default: ScopeDefault,
) -> Scope {
    let explorer = use_explorer();
    let (size_param, set_size) = use_param("size", "");
    let (rate_param, set_rate) = use_param("rate", "");
    let main = overview.provenance.main_hz();
    let ov = overview.clone();
    let rates = Memo::new(move |_| {
        let prefs = explorer.prefs.snapshot();
        Selection::new(&ov, &prefs, &scenario.get()).rates()
    });
    let hz = Memo::new(move |_| {
        let all = rates.get();
        let want: Option<f64> = rate_param.get().parse().ok();
        all.iter()
            .copied()
            .find(|r| Some(*r) == want)
            .or_else(|| all.iter().copied().find(|r| *r == main))
            .or_else(|| all.last().copied())
    });
    let ov = overview.clone();
    let sizes = Memo::new(move |_| {
        let prefs = explorer.prefs.snapshot();
        Selection::new(&ov, &prefs, &scenario.get()).sizes(hz.get())
    });
    let size = Memo::new(move |_| {
        let all = sizes.get();
        let want = size_param.get();
        let picked = all.iter().copied().find(|s| s.0.to_string() == want);
        picked.or_else(|| match default {
            ScopeDefault::Smallest => all.first().copied(),
            ScopeDefault::MostCells => {
                let prefs = explorer.prefs.snapshot();
                let sel = Selection::new(&overview, &prefs, &scenario.get());
                let hz = hz.get()?;
                all.iter()
                    .copied()
                    .max_by_key(|s| (sel.count(hz, s.0), s.0))
            }
        })
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
    /// Show the picker of the refresh rate.
    #[prop(default = true)]
    rate: bool,
    /// Show the picker of the size.
    #[prop(default = true)]
    size: bool,
) -> impl IntoView {
    let sizes = Signal::derive(move || {
        scope
            .sizes
            .try_get()
            .unwrap_or_default()
            .into_iter()
            .map(|s| (s.0.to_string(), size_label(s)))
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
        {size.then(|| view! {
            <Segmented
                options=sizes
                value=Signal::derive(move || scope.size.try_get().flatten().map(|s| s.0.to_string()).unwrap_or_default())
                on_change=scope.set_size
                title="Size"
            />
        })}
        {rate.then(|| view! {
            <Segmented
                options=rates
                value=Signal::derive(move || scope.hz.try_get().flatten().map(|h| h.to_string()).unwrap_or_default())
                on_change=scope.set_rate
                title="Refresh rate"
            />
        })}
    }
}
