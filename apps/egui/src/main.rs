//! egui under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;
mod font;
mod window;

use crate::cli::Scenario;

const NAME: &str = "egui-bench";

fn main() -> eframe::Result {
    let scenario = cli::from_env(NAME);
    let title = scenario.title();
    match scenario {
        Scenario::App { count } => dashboard::run(title, count),
        Scenario::List { count } => dashboard::list::run(title, count),
    }
}
