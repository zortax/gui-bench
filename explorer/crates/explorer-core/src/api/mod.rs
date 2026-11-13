//! The wire types of the server functions: runs, cells, metrics, provenance, the detail reports
//! of one cell and the frame times of a matrix point.

mod cell;
mod detail;
mod frames;
mod metric;
mod provenance;
mod run;

pub use cell::{CellId, CellInfo, CellKey, CheckStatus, MeasurementHealth, Part};
pub use detail::{
    CellDetail, Check, ConfigureRecord, FrameIteration, FrameStep, FramesDetail, HistogramBin,
    IterationInfo, LatencyDetail, LatencyIteration, LatencySample, Launch, MeasurementDetail,
    PacingIteration, PhaseBin, PhaseSpan, PhasesDetail, ResizeDetail, ResizeIteration, Stage,
    Summary,
};
pub use frames::{FrameProfile, FrameQuantiles, GapFrame, GapLane};
pub use metric::{Better, MetricColumn, MetricInfo, MetricRef, Stat};
pub use provenance::{
    Build, CheckTally, Effort, Environment, Flag, FrameworkVersion, Invocation, Provenance,
    Revision, ScrollSetup, ValidityTally,
};
pub use run::{RunOverview, RunStatus, RunSummary};
