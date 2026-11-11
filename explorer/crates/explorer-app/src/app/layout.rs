//! The frame of a run: the top bar, the view tabs, the sidebar, the view and the cell drawer.

use leptos::prelude::*;
use leptos_router::components::{A, Outlet};
use leptos_router::hooks::use_location;

use super::runbar::RunBar;
use super::sidebar::Sidebar;
use super::theme::ThemeSwitch;
use crate::state::provide_explorer;
use crate::views::{CellDrawer, View};
use crate::widgets::{Icon, IconKind};

/// The frame around the views of one run.
#[component]
pub fn RunLayout() -> impl IntoView {
    let explorer = provide_explorer();
    let side = RwSignal::new(true);
    view! {
        <div class="gx-app" data-side=move || side.get().then_some("")>
            <div class="gx-progress" data-on=move || explorer.pending.get().then_some("")></div>
            <header class="gx-top">
                <button
                    type="button"
                    class="gx-iconbtn"
                    title="Show or hide the filters"
                    on:click=move |_| side.update(|s| *s = !*s)
                >
                    <Icon kind=IconKind::Sidebar/>
                </button>
                <a class="gx-brand" href="/latest">"gui-bench"<span>"explorer"</span></a>
                <RunBar/>
                <span class="gx-top__fill"></span>
                <ThemeSwitch/>
            </header>
            <ViewTabs/>
            <aside class="gx-side">
                <Sidebar/>
            </aside>
            <main class="gx-main">
                <div class="gx-main__col">
                    <Outlet/>
                </div>
            </main>
            <CellDrawer/>
        </div>
    }
}

/// The tabs of the views. Links keep the query, so filters carry over.
#[component]
fn ViewTabs() -> impl IntoView {
    let explorer = crate::state::use_explorer();
    let location = use_location();
    view! {
        <nav class="gx-tabs">
            {View::ALL
                .iter()
                .map(|v| {
                    let v = *v;
                    let href = move || {
                        let search = location.search.get();
                        let search = if search.is_empty() { String::new() } else { format!("?{}", search.trim_start_matches('?')) };
                        format!("/{}/{}{search}", explorer.run.get(), v.slug())
                    };
                    view! {
                        <A href=href exact=true scroll=false>
                            <span class="gx-tabs__tab" title=v.description()>{v.label()}</span>
                        </A>
                    }
                })
                .collect_view()}
        </nav>
    }
}
