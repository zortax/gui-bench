//! Statistics for uncertainty and comparison: quantiles, seeded bootstrap intervals, effect
//! sizes, outlier fences, line fits and frame gap classes.

mod bootstrap;
mod compare;
mod fit;
mod gaps;
mod quantile;

pub use bootstrap::{Rng, median_ci, ratio_ci};
pub use compare::{Ratio, cliffs_delta, focus_limit, geomean};
pub use fit::{Fit, linear_fit};
pub use gaps::{GAP_CLASSES, GapBreakdown, GapClass, gap_breakdown, gap_class};
pub use quantile::{iqr, median, quantile, sorted};
