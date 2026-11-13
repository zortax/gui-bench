//! Serde views of the files of a run: the run index, the cell outcome, the fakeland bench report
//! (only the fields the explorer reads) and the startup report.

mod bench;
mod index;

pub use bench::{BenchReport, Environment, QuantilesRaw, Section};
pub use index::{CellOutcome, LaunchRaw, RunIndex, Startup};
