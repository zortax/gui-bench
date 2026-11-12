//! The wire types of the server functions: runs, cells, metrics, provenance, the detail reports
//! of one cell and the frame times of a matrix point.

mod cell;
mod detail;
mod frames;
mod metric;
mod provenance;
mod run;

pub use cell::{CellId, CellInfo, CellKey, CheckStatus, MeasurementHealth};
pub use detail::{
    CellDetail, Check, ConfigureRecord, FrameIteration, FrameStep, FramesDetail, HistogramBin,
    IterationInfo, KeyRecord, LatencyDetail, LatencyIteration, LatencySample, MeasurementDetail,
    PacingIteration, PhaseBin, PhaseSpan, PhasesDetail, ResizeDetail, ResizeIteration, Stage,
    Startup, Summary, TypingDetail, TypingIteration,
};
pub use frames::{FrameProfile, FrameQuantiles, GapFrame, GapLane};
pub use metric::{Better, MetricColumn, MetricInfo, MetricRef, Stat};
pub use provenance::{
    CheckTally, Effort, Environment, FrameworkVersion, Invocation, Provenance, Revision,
    ValidityTally,
};
pub use run::{RunOverview, RunStatus, RunSummary};
