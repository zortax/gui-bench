//! The explorer server: it renders the explorer pages, answers the server functions from the
//! run store, and serves the cell files (screenshots, app logs), the fonts and the site assets.

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

mod files;

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use clap::Parser;
use explorer_app::{App, shell};
use explorer_data::Store;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};

/// Serves the gui-bench explorer.
#[derive(Debug, Parser)]
struct Args {
    /// The results directory with one directory per run.
    #[arg(long)]
    results: Option<PathBuf>,
    /// The directory of the Libertinus Sans fonts.
    #[arg(long)]
    fonts: Option<PathBuf>,
}

/// Returns the gui-bench repository root.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,explorer_data=info".into()),
        )
        .init();
    let args = Args::parse();
    let results = args.results.unwrap_or_else(|| repo_root().join("results"));
    let fonts = args
        .fonts
        .unwrap_or_else(|| repo_root().join("plots/fonts"));
    let store = Arc::new(Store::new(results.canonicalize().unwrap_or(results)));
    tracing::info!(results = %store.results().display(), "run store");
    tokio::task::spawn_blocking({
        let store = store.clone();
        move || {
            if let Err(e) = store.run("latest") {
                tracing::warn!("preload of latest: {e}");
            }
        }
    });

    let conf = get_configuration(None)?;
    let options = conf.leptos_options;
    let addr = options.site_addr;
    let routes = generate_route_list(App);
    let site = PathBuf::from(options.site_root.as_ref());
    let pkg = site.join(options.site_pkg_dir.as_ref());
    // Static paths come before the routes, since `/:run/:view` would match them too.
    let app = Router::new()
        .route("/files/{run}/{cell}/{file}", get(files::serve))
        .nest_service("/fonts", ServeDir::new(fonts))
        .nest_service(&format!("/{}", options.site_pkg_dir), ServeDir::new(pkg))
        .route_service("/favicon.svg", ServeFile::new(site.join("favicon.svg")))
        .leptos_routes_with_context(
            &options,
            routes,
            {
                let store = store.clone();
                move || provide_context(store.clone())
            },
            {
                let options = options.clone();
                move || shell(options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(options)
        .layer(axum::Extension(store))
        .layer(CompressionLayer::new());
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
