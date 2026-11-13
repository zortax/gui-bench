//! Dioxus Native under gui-bench: one window per run that shows the scenario the command line
//! names. Dioxus Native renders the virtual DOM with Blitz, its own HTML and CSS engine, so every
//! scenario is markup from `rsx!` styled by a stylesheet.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;

use dioxus::prelude::*;
use dioxus_native::winit::platform::wayland::WindowAttributesWayland;
use dioxus_native::{Config, LogicalSize, WindowAttributes};

use crate::cli::Scenario;
use crate::dashboard::{Dashboard, OrderList, spec};

const NAME: &str = "dioxus-bench";
const APP_ID: &str = "gui-bench.dioxus";

fn main() {
    let scenario = cli::from_env(NAME);
    let window = WindowAttributes::default()
        .with_title(scenario.title())
        .with_surface_size(LogicalSize::new(spec::WINDOW.0, spec::WINDOW.1))
        .with_decorations(false)
        .with_platform_attributes(Box::new(
            WindowAttributesWayland::default().with_name(APP_ID, ""),
        ));
    let config = Config::new().with_window_attributes(window);
    dioxus_native::launch_cfg_with_props(
        App,
        AppProps { scenario },
        vec![],
        vec![Box::new(config)],
    );
}

/// The root component: the scenario of the command line.
#[component]
fn App(scenario: Scenario) -> Element {
    match scenario {
        Scenario::App { count } => rsx! { Dashboard { count } },
        Scenario::List { count } => rsx! { OrderList { count } },
    }
}
