//! The interval that bands and whiskers show for a statistic, by the uncertainty preference,
//! and the sentence that says so.

use explorer_core::api::Stat;
use explorer_core::stats::{quantile, sorted};

use crate::state::Uncertainty;

/// The band and whisker of one value.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Spread {
    pub band: Option<(f64, f64)>,
    pub whisker: Option<(f64, f64)>,
}

fn quantiles(values: &[f64], lo: f64, hi: f64) -> Option<(f64, f64)> {
    let s = sorted(values);
    Some((quantile(&s, lo)?, quantile(&s, hi)?))
}

/// Returns the spread of a statistic. `samples` says that its values are per-repeat samples.
pub fn spread(stat: &Stat, samples: bool, unc: Uncertainty) -> Spread {
    let range = (stat.values.len() > 1).then_some((stat.min, stat.max));
    match unc {
        Uncertainty::Auto if samples => Spread {
            band: quantiles(&stat.values, 0.25, 0.75),
            whisker: quantiles(&stat.values, 0.05, 0.95),
        },
        Uncertainty::Auto | Uncertainty::Range => Spread {
            band: None,
            whisker: range,
        },
        Uncertainty::Ci => Spread {
            band: None,
            whisker: stat.ci,
        },
        Uncertainty::Iqr => Spread {
            band: (stat.values.len() > 1)
                .then(|| quantiles(&stat.values, 0.25, 0.75))
                .flatten(),
            whisker: None,
        },
        Uncertainty::P90 => Spread {
            band: None,
            whisker: (stat.values.len() > 1)
                .then(|| quantiles(&stat.values, 0.05, 0.95))
                .flatten(),
        },
        Uncertainty::None => Spread::default(),
    }
}

/// Returns how to read a chart: what the mark, band and whiskers show.
pub fn spread_note(samples: bool, unc: Uncertainty, mark: &str) -> String {
    let center = if samples {
        "median repeat"
    } else {
        "median iteration"
    };
    let rest = match unc {
        Uncertainty::Auto if samples => "band: middle half · whiskers: 5th to 95th percentile",
        Uncertainty::Auto | Uncertainty::Range if samples => "whiskers: fastest to slowest repeat",
        Uncertainty::Auto | Uncertainty::Range => "whiskers: cheapest to most expensive iteration",
        Uncertainty::Ci if samples => "whiskers: 95% CI of the median",
        Uncertainty::Ci => "whiskers: 95% CI of the mean (3+ iterations)",
        Uncertainty::Iqr => "band: middle half",
        Uncertainty::P90 => "whiskers: 5th to 95th percentile",
        Uncertainty::None => "no interval",
    };
    format!("{mark}: {center} · {rest}")
}
