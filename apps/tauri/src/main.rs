//! Tauri under gui-bench: one window per run whose web page (`ui/`, SolidJS) shows the scenario
//! the command line names. The page asks for the scenario and the data of the spec with the
//! `spec` command when it starts.

#[path = "../../shared/cli.rs"]
mod cli;
#[path = "../../shared/dashboard.rs"]
mod dashboard;
mod payload;

use tauri::{State, WebviewUrl, WebviewWindowBuilder};

use crate::cli::Scenario;

const NAME: &str = "tauri-bench";

/// Returns the scenario and the data of the spec that the page shows.
#[tauri::command]
fn spec(scenario: State<'_, Scenario>) -> serde_json::Value {
    payload::payload(*scenario)
}

/// The variable that turns the DMA-BUF renderer of WebKitGTK off.
const DMABUF_SWITCH: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

fn main() {
    let scenario = cli::from_env(NAME);
    // On Wayland with the NVIDIA driver the DMA-BUF renderer of WebKitGTK presents 60 frames per
    // second at any refresh rate, and on some setups a blank page. Tauri apps turn it off there
    // as a matter of course; a value from the environment wins.
    if std::env::var_os(DMABUF_SWITCH).is_none() {
        // SAFETY: the process has one thread here; the web view starts its threads later.
        unsafe { std::env::set_var(DMABUF_SWITCH, "1") };
    }
    tauri::Builder::default()
        .manage(scenario)
        .invoke_handler(tauri::generate_handler![spec])
        .setup(move |app| {
            // Without decorations, so the page fills the window from its top left corner.
            WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title(scenario.title())
                .inner_size(dashboard::WINDOW.0.into(), dashboard::WINDOW.1.into())
                .decorations(false)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("the Tauri application runs");
}
