//! The wire types of the server functions: runs, cells, metrics, provenance and the detail
//! reports of one cell.

mod cell;
mod detail;
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
pub use metric::{Better, MetricColumn, MetricInfo, MetricRef, Stat};
pub use provenance::{
    CheckTally, Effort, Environment, FrameworkVersion, Provenance, Revision, ValidityTally,
};
pub use run::{RunOverview, RunStatus, RunSummary};
