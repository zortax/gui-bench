//! The explorer app: the page shell, the router, the explorer state in the URL, the server
//! functions, the chart kernel with its marks, the compact widgets and the views of a run.

#![forbid(unsafe_code)]
#![recursion_limit = "512"]

pub mod api;
mod app;
mod chart;
mod marks;
mod palette;
mod state;
mod views;
mod widgets;

pub use app::{App, shell};
