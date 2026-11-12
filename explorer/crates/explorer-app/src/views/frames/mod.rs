//! The frame-times view: frame interval percentiles against size, the frame-time profile of one
//! size, the late frames of every iteration over time, and the share of time per class of
//! lateness. Other views reuse its cards.

mod gaps;
mod profile;
mod scaling;
mod source;
mod view;

pub use gaps::GapCards;
pub use profile::ProfileCard;
pub use scaling::FrameScaling;
pub use source::{SOURCES, Source, metrics};
pub use view::FramesView;
