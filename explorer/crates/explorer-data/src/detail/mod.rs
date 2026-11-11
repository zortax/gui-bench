//! The detail loaders of one cell. Each parses one report file on demand, so the raw series
//! never sit in the run cache.

mod cell;
mod json;
mod latency;
mod series;

pub use cell::CellFile;
