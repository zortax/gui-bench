//! The chart kernel: the data models of x-y plots, dot rows, frame lanes and heatmaps, the uncertainty that
//! whiskers and bands show, the card around a chart with its toolbar and export, sizing,
//! zoom and pan, tooltips and the legend. Every chart draws SVG with colors from CSS variables.

mod card;
mod dots;
mod export;
mod heat;
mod interact;
mod lanes;
mod legend;
mod model;
mod plot;
mod size;
mod spread;

pub use card::{CardCtx, ChartCard, use_card};
pub use dots::{DotModel, DotPlot, DotRow};
pub use heat::{HeatCell, HeatModel, Heatmap};
pub use lanes::{Lane, LaneGap, LanePlot, LaneRow, LanesModel};
pub use legend::{Legend, LegendItem};
pub use model::{Axis, Bar, Pt, Rule, Series, XyModel};
pub use plot::XyPlot;
pub use spread::{Spread, spread, spread_note};
