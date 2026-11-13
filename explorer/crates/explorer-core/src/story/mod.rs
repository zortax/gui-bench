//! The story of the suite as the gui-bench plots tell it: the steps of the app scenario that
//! carry it, the frame budgets, the capacity of a framework (the size up to which a step stays
//! within a budget) and the cost of a larger app (the slope of a response over the size).

mod capacity;
mod slope;
mod steps;

pub use capacity::{Capacity, capacity, shade};
pub use slope::{SLOPE_HIGH, SLOPE_LOW, slope_per_thousand, slope_shade};
pub use steps::{
    BUDGETS, LIST_PART, PACED, RESPONSES, SMOOTH_SHARE, STOP_MS, Step, part_of, title_of,
};
