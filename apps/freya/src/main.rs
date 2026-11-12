//! freya under gui-bench: one window per run that shows the scenario the command line names.

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

const NAME: &str = "freya-bench";

fn main() {
    let scenario = cli::from_env(NAME, &[Variant::Fine, Variant::Coarse, Variant::Shared]);
    let title = scenario.title();
    match scenario {
        Scenario::Rows { count } => rows::run(count),
        Scenario::Table {
            count,
            variant,
            ticker,
        } => table::run(scenario, count, variant, ticker),
        Scenario::Spinner { count } => spinner::run(title, count),
        Scenario::Search { count, filter } => search::run(title, count, filter),
        Scenario::Virtual { count } => virtual_rows::run(title, count),
        Scenario::Dashboard { count } => dashboard::run(title, count),
    }
}
