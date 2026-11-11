//! The data side of the explorer: the catalog of runs in a results directory, the parsing of run
//! indexes and fakeland bench reports, a cache of loaded runs with their metric columns, and the
//! loaders of the raw series of one cell. It runs on the server only.

#![forbid(unsafe_code)]

mod catalog;
mod detail;
mod error;
mod report;
mod store;

pub use detail::CellFile;
pub use error::DataError;
pub use store::{RunData, Store};
