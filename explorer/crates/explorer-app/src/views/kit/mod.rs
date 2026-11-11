//! The shared parts of the views: loading metric columns with skeletons and errors, building
//! series and rows from columns (with uncertainty, baseline, validity and the compare run),
//! and compact view controls.

mod build;
mod controls;
mod data;
mod dots;

pub use build::{Build, baseline_label, cell_tip, ecdf};
pub use controls::{SizePicker, ViewHead, sizes_label};
pub use data::{Data, WithData};
pub use dots::{DotsCard, ScopePicker};
