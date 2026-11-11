//! The x-y plot: its scene (scales, ticks and pixel geometry), its SVG layers, and the
//! component with hover, tooltips, zoom and pan.

mod component;
mod layers;
mod scene;
mod tooltip;

pub use component::XyPlot;
