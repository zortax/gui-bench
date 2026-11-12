//! Dioxus Native under gui-bench: one window per run that shows the scenario the command line
//! names. Dioxus Native renders the virtual DOM with Blitz, its own HTML and CSS engine, so every
//! scenario is markup from `rsx!` styled by a stylesheet.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;
mod page;
mod rows;
mod search;
#[path = "../../shared/spec.rs"]
mod spec;
mod spinner;
mod table;
mod virtual_rows;

use dioxus::prelude::*;
use dioxus_native::winit::platform::wayland::WindowAttributesWayland;
use dioxus_native::{Config, LogicalSize, WindowAttributes};

use crate::cli::{Scenario, Variant};
use crate::dashboard::Dashboard;
use crate::rows::Rows;
use crate::search::Search;
use crate::spinner::Spinner;
use crate::table::{CoarseTable, FineTable};
use crate::virtual_rows::VirtualRows;

const NAME: &str = "dioxus-bench";
const APP_ID: &str = "gui-bench.dioxus";

fn main() {
    let scenario = cli::from_env(NAME, &[Variant::Fine, Variant::Coarse, Variant::Shared]);
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
        Scenario::Rows { count } => rsx! { Rows { count } },
        Scenario::Table {
            count,
            variant: Variant::Coarse,
            ticker,
        } => rsx! { CoarseTable { count, ticker } },
        Scenario::Table {
            count,
            variant,
            ticker,
        } => rsx! { FineTable { count, ticker, shared: variant == Variant::Shared } },
        Scenario::Spinner { count } => rsx! { Spinner { count } },
        Scenario::Search { count, filter } => rsx! { Search { count, filter } },
        Scenario::Virtual { count } => rsx! { VirtualRows { count } },
        Scenario::Dashboard { count } => rsx! { Dashboard { count } },
    }
}
