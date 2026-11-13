//! vizia under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;
#[path = "../../shared/dashboard.rs"]
mod dashboard_spec;

use vizia::prelude::*;

use crate::cli::Scenario;
use crate::dashboard_spec::WINDOW;

const NAME: &str = "vizia-bench";

fn main() -> Result<(), ApplicationError> {
    let scenario = cli::from_env(NAME);
    Application::new(move |cx| {
        cx.emit(EnvironmentEvent::SetThemeMode(ThemeMode::LightMode));
        match scenario {
            Scenario::App { count } => dashboard::build(cx, count),
            Scenario::List { count } => dashboard::list::build(cx, count),
        }
    })
    .title(scenario.title())
    .inner_size((WINDOW.0 as u32, WINDOW.1 as u32))
    .decorations(false)
    .run()
}
