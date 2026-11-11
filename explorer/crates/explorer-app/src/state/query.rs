//! Writes to the URL query. Preference changes replace the history entry and keep the scroll
//! position.

use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_location, use_navigate};

/// Sets or removes query parameters of the current URL.
#[derive(Clone, Copy)]
pub struct QueryWriter(Callback<Vec<(&'static str, Option<String>)>>);

impl QueryWriter {
    /// Returns a writer for the current router.
    pub fn new() -> Self {
        let navigate = use_navigate();
        let location = use_location();
        Self(Callback::new(
            move |changes: Vec<(&'static str, Option<String>)>| {
                let mut query = location.query.get_untracked();
                for (key, value) in changes {
                    query.remove(key);
                    if let Some(value) = value.filter(|v| !v.is_empty()) {
                        query.insert(key, value);
                    }
                }
                let path = location.pathname.get_untracked();
                let search = query.to_query_string();
                navigate(
                    &format!("{path}{search}"),
                    NavigateOptions {
                        replace: true,
                        scroll: false,
                        ..Default::default()
                    },
                );
            },
        ))
    }

    /// Sets one parameter; `None` or an empty value removes it.
    pub fn set(&self, key: &'static str, value: Option<String>) {
        self.0.run(vec![(key, value)]);
    }

    /// Sets several parameters at once.
    pub fn set_all(&self, changes: Vec<(&'static str, Option<String>)>) {
        self.0.run(changes);
    }
}

/// Returns a view parameter of the URL query with a default, and its setter. The default stays
/// out of the URL.
pub fn use_param(key: &'static str, default: &'static str) -> (Memo<String>, Callback<String>) {
    let query = leptos_router::hooks::use_query_map();
    let writer = QueryWriter::new();
    let value = Memo::new(move |_| {
        query
            .with(|q| q.get(key))
            .unwrap_or_else(|| default.to_owned())
    });
    let set = Callback::new(move |v: String| writer.set(key, (v != default).then_some(v)));
    (value, set)
}
