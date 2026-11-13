//! floem under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
#[path = "../../shared/dashboard.rs"]
mod dash;
mod dashboard;

use floem::kurbo::Size;
use floem::prelude::*;
use floem::window::{Theme, WindowConfig};
use floem::{AnyView, Application};

use crate::cli::Scenario;

const NAME: &str = "floem-bench";

fn main() {
    let scenario = cli::from_env(NAME);
    let config = WindowConfig::default()
        .title(scenario.title())
        .size(Size::new(
            f64::from(dash::WINDOW.0),
            f64::from(dash::WINDOW.1),
        ))
        .show_titlebar(false)
        .undecorated(true)
        .theme_override(Theme::Light);
    Application::new()
        .window(move |_| content(scenario), Some(config))
        .run();
}

/// Returns the window content of a scenario.
fn content(scenario: Scenario) -> AnyView {
    match scenario {
        Scenario::App { count } => dashboard::view(count).into_any(),
        Scenario::List { count } => dashboard::list::view(count).into_any(),
    }
}
