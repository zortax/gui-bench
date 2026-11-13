//! zgui under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;

use zgui::prelude::*;

use crate::cli::Scenario;
use crate::dashboard::{DashboardProps, OrderListProps, spec};

const NAME: &str = "zgui-bench";

fn main() -> Result<(), zgui::Error> {
    let scenario = cli::from_env(NAME);
    let window = app()
        .with_application_id("gui-bench.zgui")
        .with_title(scenario.title())
        .with_size(spec::WINDOW.0, spec::WINDOW.1)
        .with_stylesheet(dashboard::SHEET);
    match scenario {
        Scenario::App { count } => window.run(move || view! { Dashboard(count = count) }),
        Scenario::List { count } => window.run(move || view! { OrderList(count = count) }),
    }
}
