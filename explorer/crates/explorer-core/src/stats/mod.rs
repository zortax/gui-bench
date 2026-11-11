//! Statistics for uncertainty and comparison: quantiles, seeded bootstrap intervals, effect
//! sizes, outlier fences and line fits.

mod bootstrap;
mod compare;
mod fit;
mod quantile;

pub use bootstrap::{Rng, median_ci, ratio_ci};
pub use compare::{Ratio, cliffs_delta, focus_limit, geomean};
pub use fit::{Fit, linear_fit};
pub use quantile::{iqr, median, quantile, sorted};
