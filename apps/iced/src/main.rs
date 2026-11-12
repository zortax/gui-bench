//! iced under gui-bench: one window per run that shows the scenario the command line names.

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

use crate::cli::{Scenario, Variant};

const NAME: &str = "iced-bench";
const APP_ID: &str = "gui-bench.iced";

fn main() -> iced::Result {
    let scenario = cli::from_env(NAME, &[Variant::Fine, Variant::Coarse]);
    let title = scenario.title();
    match scenario {
        Scenario::Rows { count } => rows::run(count, APP_ID),
        Scenario::Table {
            count,
            variant,
            ticker,
        } => table::run(count, variant, ticker, title, APP_ID),
        Scenario::Spinner { count } => spinner::run(count, title, APP_ID),
        Scenario::Search { count, filter } => search::run(count, filter, title, APP_ID),
        Scenario::Virtual { count } => virtual_rows::run(count, title, APP_ID),
        Scenario::Dashboard { count } => dashboard::run(count, title, APP_ID),
    }
}
