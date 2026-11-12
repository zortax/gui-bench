//! zgui under gui-bench: one window per run that shows the scenario the command line names.

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

use zgui::prelude::*;

use crate::cli::{Scenario, Variant};
use crate::dashboard::DashboardProps;
use crate::rows::RowsProps;
use crate::search::SearchProps;
use crate::spinner::SpinnerProps;
use crate::table::{CoarseTableProps, FineTableProps};
use crate::virtual_rows::VirtualRowsProps;

const NAME: &str = "zgui-bench";

fn main() -> Result<(), zgui::Error> {
    let scenario = cli::from_env(NAME, &[Variant::Fine, Variant::Coarse, Variant::Shared]);
    let window = app()
        .with_application_id("gui-bench.zgui")
        .with_title(scenario.title())
        .with_size(spec::WINDOW.0, spec::WINDOW.1);
    match scenario {
        Scenario::Rows { count } => window
            .with_stylesheet(rows::SHEET)
            .run(move || view! { Rows(count = count) }),
        Scenario::Virtual { count } => window
            .with_stylesheet(rows::SHEET)
            .run(move || view! { VirtualRows(count = count) }),
        Scenario::Table {
            count,
            variant,
            ticker,
        } => window.with_stylesheet(page::SHEET).run(move || match variant {
            Variant::Coarse => AnyView::new(view! { CoarseTable(count = count, ticker = ticker) }),
            v => AnyView::new(
                view! { FineTable(count = count, ticker = ticker, shared = v == Variant::Shared) },
            ),
        }),
        Scenario::Spinner { count } => window
            .with_stylesheet(page::SHEET)
            .run(move || view! { Spinner(count = count) }),
        Scenario::Search { count, filter } => window
            .with_stylesheet(page::SHEET)
            .run(move || view! { Search(count = count, filter = filter) }),
        Scenario::Dashboard { count } => window
            .with_stylesheet(dashboard::SHEET)
            .run(move || view! { Dashboard(count = count) }),
    }
}
