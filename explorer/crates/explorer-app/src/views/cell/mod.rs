//! The cell drawer: everything about one matrix cell, opened from any chart point. It shows the
//! screenshot, the startup, the validity checks and iterations of every measurement, the raw
//! series (latency repeats, frames, keys, configures, app phases) and the app log.

mod drawer;
mod latency;
mod measurement;
mod series;

pub use drawer::CellDrawer;
