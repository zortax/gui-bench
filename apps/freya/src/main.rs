//! freya under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;

use crate::cli::Scenario;

const NAME: &str = "freya-bench";

fn main() {
    let scenario = cli::from_env(NAME);
    let title = scenario.title();
    match scenario {
        Scenario::App { count } => dashboard::run_app(title, count),
        Scenario::List { count } => dashboard::run_list(title, count),
    }
}
