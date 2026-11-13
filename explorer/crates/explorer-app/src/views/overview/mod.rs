//! The overview of a run: how far every framework scales within a frame budget, its provenance,
//! its health, and its builds with the parts built by hand.

mod builds;
mod capacity;
mod health;
mod view;

pub use view::OverviewView;
