//! A window with one scroll container that holds N real rows, for framework benchmarks.

#![forbid(unsafe_code)]

mod cli;
mod row;
mod rows;
mod wheel;

use zgui::prelude::*;

use crate::rows::RowsProps;

/// The look of the window, the scroll container and every row.
const SHEET: &str = include_str!("rows.css");

fn main() -> Result<(), zgui::Error> {
    let count = match cli::rows_from(std::env::args().skip(1)) {
        Ok(count) => count,
        Err(error) => {
            eprintln!("zgui-rows: {error}");
            eprintln!("usage: zgui-rows [--rows N]");
            std::process::exit(2);
        }
    };
    eprintln!("zgui-rows: {count} rows");
    app()
        .with_application_id("gui-bench.zgui")
        .with_title("gui-bench rows")
        .with_size(1280.0, 800.0)
        .with_stylesheet(SHEET)
        .run(move || view! { Rows(count = count) })
}
