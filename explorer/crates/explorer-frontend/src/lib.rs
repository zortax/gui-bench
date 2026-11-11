//! The hydration entry of the explorer: it attaches the app to the server-rendered page.

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

/// Hydrates the server-rendered body.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(explorer_app::App);
}
