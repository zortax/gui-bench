//! The compare view: one framework against another, or the run against the compare
//! run, over every matching cell of the key metrics or of every metric, as a forest plot of
//! ratios with bootstrap intervals.

mod forest;
mod view;

pub use view::CompareView;
