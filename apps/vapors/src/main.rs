//! Vapors implementation of the shared orders dashboard.

#![allow(
    clippy::missing_panics_doc,
    reason = "a generated mount of MessageCard panics on a colour outside AVATAR_COLORS, as it must, and the generated function has no doc to put `# Panics` in"
)]

mod app;
#[allow(clippy::all, clippy::pedantic, dead_code)]
#[path = "../../shared/cli.rs"]
mod cli;
mod controls;
#[allow(clippy::all, clippy::pedantic, dead_code)]
#[path = "../../shared/dashboard.rs"]
mod dashboard;
mod live;
mod order_parts;
include!(concat!(env!("OUT_DIR"), "/wisp.rs"));
#[cfg(test)]
mod tests;

fn main() {
    let scenario = cli::from_env("vapors-bench");
    if let Err(message) = app::run(scenario) {
        eprintln!("vapors-bench: {message}");
        std::process::exit(2);
    }
}
