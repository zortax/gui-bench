//! gpui-kit under gui-bench: one window per run that shows the scenario the command line names.

#[path = "../../shared/cli.rs"]
mod cli;
mod dashboard;

use gpui_kit::*;

use crate::cli::Scenario;
use crate::dashboard::{Dashboard, OrderList, spec};

const NAME: &str = "gpui-kit-bench";
const APP_ID: &str = "gui-bench.gpui-kit";

fn main() {
    let scenario = cli::from_env(NAME);
    // `apps/gpui-kit-fast` builds the same sources with the `gpui-fast` feature.
    eprintln!("{NAME}: gpui-fast {}", gpui_kit::GPUI_FAST);

    // The kit's component icons, such as the check mark of the checkbox.
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    app.run(move |cx| {
        gpui_kit::init(cx);
        dashboard::apply_theme(cx);
        let bounds = Bounds::centered(None, size(px(spec::WINDOW.0), px(spec::WINDOW.1)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some(scenario.title().into()),
                ..Default::default()
            }),
            app_id: Some(APP_ID.into()),
            window_decorations: Some(WindowDecorations::Server),
            ..Default::default()
        };
        let opened = match scenario {
            Scenario::App { count } => gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| Dashboard::new(count, window, cx))
            })
            .map(drop),
            Scenario::List { count } => {
                gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| OrderList::new(count)))
                    .map(drop)
            }
        };
        opened.expect("failed to open window");
        cx.activate(true);
    });
}
