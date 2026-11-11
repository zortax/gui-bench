//! gpui-kit under gui-bench: one window per run that shows the scenario the command line names.

#[path = "../../shared/cli.rs"]
mod cli;
mod page;
mod rows;
mod search;
#[path = "../../shared/spec.rs"]
mod spec;
mod spinner;
mod table;
mod virtual_rows;

use gpui_kit::*;

use crate::cli::{Scenario, Variant};
use crate::rows::Rows;
use crate::search::Search;
use crate::spinner::Spinner;
use crate::table::{CoarseTable, FineTable};
use crate::virtual_rows::VirtualRows;

const NAME: &str = "gpui-kit-bench";
const APP_ID: &str = "gui-bench.gpui-kit";

fn main() {
    let scenario = cli::from_env(NAME, &[Variant::Fine, Variant::Coarse]);
    // The same sources build `gpui-kit-fast-bench` with the `gpui-fast` feature.
    eprintln!("{NAME}: gpui-fast {}", gpui_kit::GPUI_FAST);

    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        table::keys::bind(cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(scenario.title().into()),
                ..Default::default()
            }),
            app_id: Some(APP_ID.into()),
            window_decorations: Some(WindowDecorations::Server),
            ..Default::default()
        };
        let opened = match scenario {
            Scenario::Rows { count } => {
                gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| Rows::new(count))).map(drop)
            }
            Scenario::Virtual { count } => {
                gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| VirtualRows::new(count)))
                    .map(drop)
            }
            Scenario::Table {
                count,
                variant: Variant::Coarse,
                ticker,
            } => gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| CoarseTable::new(count, ticker, window, cx))
            })
            .map(drop),
            Scenario::Table { count, ticker, .. } => {
                gpui_kit::open_window(options, cx, |window, cx| {
                    cx.new(|cx| FineTable::new(count, ticker, window, cx))
                })
                .map(drop)
            }
            Scenario::Spinner { count } => {
                gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| Spinner::new(count)))
                    .map(drop)
            }
            Scenario::Search { count, filter } => {
                gpui_kit::open_window(options, cx, |window, cx| {
                    cx.new(|cx| Search::new(count, filter, window, cx))
                })
                .map(drop)
            }
        };
        opened.expect("failed to open window");
        cx.activate(true);
    });
}
