//! Benchmark app: one window with a vertical scroll container of N real rows.

mod args;
mod rows;

use gpui_kit::*;

use crate::rows::Rows;

const APP_ID: &str = "gui-bench.gpui-kit";
const TITLE: &str = "gui-bench rows";

fn main() {
    let rows = match args::parse() {
        Ok(rows) => rows,
        Err(message) => {
            eprintln!("gpui-kit-rows: {message}");
            eprintln!("usage: gpui-kit-rows [--rows N]");
            std::process::exit(2);
        }
    };
    eprintln!("gpui-kit-rows: {rows} rows");

    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(TITLE.into()),
                ..Default::default()
            }),
            app_id: Some(APP_ID.into()),
            window_decorations: Some(WindowDecorations::Server),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| Rows::new(rows)))
            .expect("failed to open window");
        cx.activate(true);
    });
}
