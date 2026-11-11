//! The scale and tick math of the charts: linear and log scales with nice ticks, zoom and pan
//! in data space, and the spacing of direct labels.

mod labels;
mod mapping;
mod ticks;

pub use labels::spread;
pub use mapping::{Scale, ScaleKind, padded};
pub use ticks::{Tick, linear_ticks, log_ticks};
