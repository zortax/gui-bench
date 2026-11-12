//! vizia under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;
#[path = "../../shared/dashboard.rs"]
mod dashboard_spec;
mod page;
mod rows;
mod search;
#[path = "../../shared/spec.rs"]
mod spec;
mod spinner;
mod table;
mod virtual_rows;

use vizia::prelude::*;

use crate::cli::{Scenario, Variant};

const NAME: &str = "vizia-bench";

fn main() -> Result<(), ApplicationError> {
    let scenario = cli::from_env(NAME, &[Variant::Fine, Variant::Coarse, Variant::Shared]);
    Application::new(move |cx| {
        cx.emit(EnvironmentEvent::SetThemeMode(ThemeMode::LightMode));
        match scenario {
            Scenario::Rows { count } => rows::build(cx, count),
            Scenario::Virtual { count } => virtual_rows::build(cx, count),
            Scenario::Table {
                count,
                variant,
                ticker,
            } => table::build(cx, count, variant, ticker),
            Scenario::Spinner { count } => spinner::build(cx, count),
            Scenario::Search { count, filter } => search::build(cx, count, filter),
            Scenario::Dashboard { count } => dashboard::build(cx, count),
        }
    })
    .title(scenario.title())
    .inner_size((spec::WINDOW.0 as u32, spec::WINDOW.1 as u32))
    .decorations(false)
    .run()
}
