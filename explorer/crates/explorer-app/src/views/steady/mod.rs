//! The steady view: what continuous animation costs (ticker and spinner): CPU and GPU time per
//! second, delivered frames, damage and wakeups, and the efficiency matrix of every scenario.

mod efficiency;
mod view;

pub use view::SteadyView;
