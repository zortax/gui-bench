//! The smoothness view: the slowest frames and the missed refreshes of scrolling and animation,
//! and the resize, against the size of the app; and the fling frame by frame at one size.

mod charts;
mod view;

pub use view::SmoothnessView;
