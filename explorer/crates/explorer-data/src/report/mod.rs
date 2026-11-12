//! Serde views of the files of a run: the run index, the cell outcome, the fakeland bench report
//! (only the fields the explorer reads), the startup report and the churn snapshot.

mod bench;
mod index;

pub use bench::{BenchReport, Environment, QuantilesRaw, Section};
pub use index::{CellOutcome, RunIndex, Startup};
