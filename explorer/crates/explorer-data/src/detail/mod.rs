//! The detail loaders: one cell, or the frame profiles of one matrix point. Each parses report
//! files on demand, so the raw series never sit in the run cache.

mod cell;
mod frames;
mod json;
mod latency;
mod series;

pub use cell::CellFile;
pub use frames::Point;
