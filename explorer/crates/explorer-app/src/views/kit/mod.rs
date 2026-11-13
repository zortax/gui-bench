//! The shared parts of the views: loading metric columns with skeletons and errors, building
//! series and rows from columns (with uncertainty, baseline, validity, hand-built parts and the
//! compare run), the size and rate a view looks at, the legend of a scenario and compact view
//! controls.

mod build;
mod controls;
mod data;
mod dots;
mod legend;
mod scope;
mod story;

pub use build::{Build, baseline_label, cell_tip};
pub use controls::{ViewHead, size_axis, size_words};
pub use data::{Data, WithData};
pub use dots::ScopeDotsCard;
pub use legend::{NoCells, StoryLegend};
pub use scope::{Scope, ScopeBar, ScopeDefault, use_scope};
pub use story::{
    APP, LIST, all_sizes, app_series, missed, missed_metrics, response, response_metrics, slowest,
    wrap,
};
