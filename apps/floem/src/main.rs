//! floem under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
#[path = "../../shared/dashboard.rs"]
mod dash;
mod dashboard;
mod page;
mod rows;
mod search;
#[path = "../../shared/spec.rs"]
mod spec;
mod spinner;
mod table;
mod virtual_rows;

use floem::kurbo::Size;
use floem::prelude::*;
use floem::window::{Theme, WindowConfig};
use floem::{AnyView, Application};

use crate::cli::{Scenario, Variant};

const NAME: &str = "floem-bench";

fn main() {
    let scenario = cli::from_env(NAME, &[Variant::Fine, Variant::Coarse, Variant::Shared]);
    let config = WindowConfig::default()
        .title(scenario.title())
        .size(Size::new(
            f64::from(spec::WINDOW.0),
            f64::from(spec::WINDOW.1),
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
        Scenario::Rows { count } => rows::view(count).into_any(),
        Scenario::Virtual { count } => virtual_rows::view(count).into_any(),
        Scenario::Table {
            count,
            variant: Variant::Coarse,
            ticker,
        } => table::coarse::view(count, ticker).into_any(),
        Scenario::Table {
            count,
            variant,
            ticker,
        } => table::fine::view(count, ticker, variant == Variant::Shared).into_any(),
        Scenario::Spinner { count } => spinner::view(count).into_any(),
        Scenario::Search { count, filter } => search::view(count, filter).into_any(),
        Scenario::Dashboard { count } => dashboard::view(count).into_any(),
    }
}
