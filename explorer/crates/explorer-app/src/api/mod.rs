//! The data the views read: the server functions and a client cache of metric columns.

mod cache;
mod server;

pub use cache::{fetch_columns, metric_list, remember};
pub use server::*;
