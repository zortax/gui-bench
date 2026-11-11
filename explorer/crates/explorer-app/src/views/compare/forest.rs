//! The forest plot rows: the ratio of A over B per metric and matching cell, with the 95%
//! bootstrap interval, colored by which side is better when the interval excludes 1.

use explorer_core::api::{Better, CellInfo, CellKey, MetricRef, Stat};
use explorer_core::stats::{Ratio, ratio_ci};
use explorer_core::units::short;

use crate::chart::DotRow;

/// The verdict of one comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    ABetter,
    BBetter,
    Same,
}

/// One compared metric in one cell pair.
#[derive(Debug, Clone)]
pub struct Pair<'a> {
    pub metric: MetricRef,
    pub better: Better,
    pub unit: String,
    pub a: &'a CellInfo,
    pub sa: Stat,
    pub sb: Stat,
}

/// Returns the ratio of a pair with its interval.
pub fn ratio(p: &Pair) -> Option<Ratio> {
    if p.sa.median <= 0.0 || p.sb.median <= 0.0 {
        return None;
    }
    ratio_ci(&p.sa.values, &p.sb.values, 0.95)
}

/// The ratio beyond which a comparison without an interval counts as a difference.
const NO_CI_MARGIN: f64 = 1.1;

/// Returns the verdict of a ratio: clear when its interval excludes 1, or, without an interval
/// (a single value per side), when it lies beyond a 10 % margin.
pub fn verdict(r: &Ratio, better: Better) -> Verdict {
    let clear = match r.lo.zip(r.hi) {
        Some((lo, hi)) => lo > 1.0 || hi < 1.0,
        None => r.estimate > NO_CI_MARGIN || r.estimate < 1.0 / NO_CI_MARGIN,
    };
    if !clear || better == Better::Neither {
        return Verdict::Same;
    }
    match (better, r.estimate < 1.0) {
        (Better::Lower, true) | (Better::Higher, false) => Verdict::ABetter,
        _ => Verdict::BBetter,
    }
}

/// Returns the short name of a metric, such as `hover total_ms`.
pub fn metric_name(m: &MetricRef) -> String {
    format!("{} {}", m.measurement, m.metric)
}

/// Returns the forest row of a pair.
pub fn row(p: &Pair, r: Ratio, a_label: &str, b_label: &str) -> DotRow {
    let v = verdict(&r, p.better);
    let label = format!(
        "{} · {} · {} Hz",
        metric_name(&p.metric),
        size(&p.a.key),
        p.a.key.hz
    );
    let mut row = DotRow::of_framework(format!("{}|{}", p.metric, p.a.id), "", label);
    let (color, soft) = match v {
        Verdict::ABetter => (
            "var(--div-lo)",
            "color-mix(in oklab, var(--div-lo) 35%, var(--surface))",
        ),
        Verdict::BBetter => (
            "var(--div-hi)",
            "color-mix(in oklab, var(--div-hi) 35%, var(--surface))",
        ),
        Verdict::Same => ("var(--muted)", "var(--rule)"),
    };
    row.color = color.to_owned();
    row.soft = soft.to_owned();
    row.ink = "var(--text-2)".to_owned();
    row.group = p.metric.measurement.clone();
    // The plotted ratio leans so that left of 1 always means A is better: metrics whose larger
    // values are better flip.
    let flip = p.better == Better::Higher;
    let lean = |v: f64| if flip { 1.0 / v } else { v };
    let estimate = lean(r.estimate);
    let interval = r.lo.zip(r.hi).map(|(lo, hi)| {
        let (a, b) = (lean(lo), lean(hi));
        (a.min(b), a.max(b))
    });
    row.value = Some(estimate);
    row.whisker = interval;
    row.hollow = v == Verdict::Same;
    row.cell = Some(p.a.id);
    let side = |v: f64| {
        if v < 1.0 {
            format!("{}× {a_label}", short(1.0 / v))
        } else {
            format!("{}× {b_label}", short(v))
        }
    };
    let no_ci = if r.lo.is_none() { " · no CI" } else { "" };
    row.text = match v {
        Verdict::ABetter | Verdict::BBetter => format!("{} better{no_ci}", side(estimate)),
        Verdict::Same => format!("≈ {}×{no_ci}", short(factor(estimate))),
    };
    row.tip = vec![
        (
            a_label.to_owned(),
            format!("{} {}", short(p.sa.median), p.unit),
        ),
        (
            b_label.to_owned(),
            format!("{} {}", short(p.sb.median), p.unit),
        ),
        ("advantage".to_owned(), format!("{} better", side(estimate))),
        (
            "95% CI".to_owned(),
            interval.map_or("–".to_owned(), |(lo, hi)| {
                format!("{} … {}", side(lo), side(hi))
            }),
        ),
        ("n".to_owned(), format!("{} vs {}", p.sa.n(), p.sb.n())),
        (
            "better".to_owned(),
            format!("{:?} values", p.better).to_lowercase(),
        ),
    ];
    row
}

/// Returns a ratio as a factor of at least 1, such as 2 for 0.5.
pub fn factor(v: f64) -> f64 {
    if v < 1.0 && v > 0.0 { 1.0 / v } else { v }
}

fn size(k: &CellKey) -> String {
    let rows = explorer_core::units::thousands(f64::from(k.rows));
    match k.updates {
        Some(u) => format!("{rows}×{u}"),
        None => rows,
    }
}
