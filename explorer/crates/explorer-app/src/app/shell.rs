//! The HTML document and the router. The document sets the stored theme before the first paint,
//! so a dark page never flashes light.

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{ParentRoute, Redirect, Route, Router, Routes};
use leptos_router::path;

use super::layout::RunLayout;
use crate::views::ViewHost;

/// Sets `data-theme` from local storage before the body renders.
const THEME_SCRIPT: &str = "try{var t=localStorage.getItem('gx-theme');if(t==='light'||t==='dark')document.documentElement.dataset.theme=t}catch(e){}";

/// The HTML document around the app.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="color-scheme" content="light dark"/>
                <script>{THEME_SCRIPT}</script>
                <link rel="preload" href="/fonts/LibertinusSans-Regular.ttf" r#as="font" crossorigin=""/>
                <link rel="icon" href="/favicon.svg" type="image/svg+xml"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// The explorer app.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/explorer.css"/>
        <Title text="gui-bench explorer"/>
        <Router>
            <Routes fallback=|| view! { <p class="gx-notfound">"No such page."</p> }>
                <Route path=path!("/") view=|| view! { <Redirect path="/latest"/> }/>
                <ParentRoute path=path!("/:run") view=RunLayout>
                    <Route path=path!("") view=ViewHost/>
                    <Route path=path!(":view") view=ViewHost/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
