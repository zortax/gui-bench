//! The window of every scenario: a borderless 1280x800 eframe window with the scenario's title.

use eframe::egui::{Context, ViewportBuilder};

use crate::spec;

const APP_ID: &str = "gui-bench.egui";

/// Opens the window, styles its context with `setup`, shows the app that `app` returns and runs
/// until the window closes.
pub fn run(
    title: &str,
    setup: fn(&Context),
    app: impl FnOnce() -> Box<dyn eframe::App> + 'static,
) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_title(title)
            .with_inner_size([spec::WINDOW.0, spec::WINDOW.1])
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        title,
        options,
        Box::new(move |cc| {
            if let Some(render) = &cc.wgpu_render_state {
                let info = render.adapter.get_info();
                eprintln!("egui-bench: wgpu {:?} on {}", info.backend, info.name);
            }
            setup(&cc.egui_ctx);
            Ok(app())
        }),
    )
}
