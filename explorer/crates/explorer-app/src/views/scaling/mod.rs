//! The rows view: hover latency, scroll and startup against the row count, for the list with
//! every row built (`rows`) and the virtualized list (`virtual`).

mod charts;
mod view;

pub use view::ScalingView;
