//! iced under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;

use crate::cli::Scenario;

const NAME: &str = "iced-bench";
const APP_ID: &str = "gui-bench.iced";

fn main() -> iced::Result {
    let scenario = cli::from_env(NAME);
    let title = scenario.title();
    match scenario {
        Scenario::App { count } => dashboard::run(count, title, APP_ID),
        Scenario::List { count } => dashboard::list::run(count, title, APP_ID),
    }
}
