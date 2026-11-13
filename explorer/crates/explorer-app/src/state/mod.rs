//! The explorer state: the run and its overview, and the preferences that live in the URL query
//! (filters, scales, uncertainty, focus, baseline, compare run, open cell), so every view links.

mod explorer;
mod prefs;
mod query;
mod selection;

pub use explorer::{Explorer, RunView, provide_explorer, use_explorer};
pub use prefs::{Prefs, PrefsSnapshot, ScalePref, Uncertainty};
pub use query::{QueryWriter, use_param};
pub use selection::{Selection, Size, cells_of, stop_of};
