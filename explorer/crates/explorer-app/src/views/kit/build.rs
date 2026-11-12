//! Builders that turn metric columns into chart series and rows: one point or row per shown
//! cell, with the spread of the uncertainty preference, the baseline ratio, the validity of the
//! report, and the compare run as ghosts.

use explorer_core::api::{CellInfo, CellKey, MetricRef, Stat};
use explorer_core::units::{display, short};

use super::data::Data;
use crate::chart::{DotRow, Pt, Series, Spread, spread};
use crate::palette::{self, Marker};
use crate::state::{PrefsSnapshot, Uncertainty, cells_of};

/// The inputs of every builder.
pub struct Build<'a> {
    pub data: &'a Data,
    pub prefs: &'a PrefsSnapshot,
    pub unc: Uncertainty,
}

/// A value of a cell in display units, with its spread and statistic.
#[derive(Debug, Clone)]
pub struct Value {
    pub y: f64,
    pub spread: Spread,
    pub stat: Stat,
    pub invalid: bool,
}

/// Returns the axis words of a metric: the baseline ratio, or the unit words.
pub fn baseline_label(prefs: &PrefsSnapshot, unit: &str) -> String {
    match &prefs.base {
        Some(base) => format!("× {base}"),
        None => display(unit).words.to_owned(),
    }
}

/// Returns tooltip lines about a cell and its statistic.
pub fn cell_tip(cell: &CellInfo, stat: &Stat, samples: bool) -> Vec<(String, String)> {
    let n = stat.n();
    let what = if samples { "repeats" } else { "iterations" };
    vec![
        ("n".to_owned(), format!("{n} {what}")),
        (
            "cell".to_owned(),
            format!("{} · {} Hz", cell.key.size(), cell.key.hz),
        ),
    ]
}

/// Returns the step points of the empirical CDF of values.
pub fn ecdf(values: &[f64]) -> Vec<Pt> {
    let s = explorer_core::stats::sorted(values);
    let n = s.len() as f64;
    let mut out = Vec::with_capacity(s.len() + 1);
    if let Some(first) = s.first() {
        out.push(Pt::new(*first, 0.0));
    }
    for (i, v) in s.iter().enumerate() {
        out.push(Pt::new(*v, (i + 1) as f64 / n));
    }
    out
}

fn same_size(a: &CellKey, b: &CellKey) -> bool {
    a.scenario == b.scenario && a.hz == b.hz && a.rows == b.rows && a.updates == b.updates
}

impl<'a> Build<'a> {
    pub fn new(data: &'a Data, prefs: &'a PrefsSnapshot, unc: Uncertainty) -> Self {
        Self { data, prefs, unc }
    }

    /// Returns the shown cells of a scenario in plot order.
    pub fn cells(&self, scenario: &str) -> Vec<&'a CellInfo> {
        cells_of(&self.data.run.overview, self.prefs, scenario)
    }

    /// Returns the median of the baseline framework at the size of a cell: the same variant
    /// when the baseline has it, else its first variant.
    pub fn baseline(&self, m: &MetricRef, key: &CellKey) -> Option<f64> {
        let base = self.prefs.base.as_ref()?;
        let candidates: Vec<&CellInfo> = self
            .data
            .overview()
            .cells
            .iter()
            .filter(|c| c.key.framework == *base && same_size(&c.key, key))
            .collect();
        let cell = candidates
            .iter()
            .find(|c| c.key.variant == key.variant)
            .or_else(|| {
                candidates
                    .iter()
                    .min_by_key(|c| palette::variant_rank(&c.key.variant))
            })?;
        self.data
            .stat(m, cell.id)
            .map(|s| s.median)
            .filter(|v| *v != 0.0)
    }

    /// Returns the display value of a metric in a cell, rebased on the baseline when one is set.
    /// Invalid reports drop out under the valid-only preference.
    pub fn value(&self, m: &MetricRef, cell: &CellInfo) -> Option<Value> {
        let stat = self.data.stat(m, cell.id)?;
        let invalid = !cell.is_valid(&m.measurement);
        if invalid && self.prefs.valid_only {
            return None;
        }
        let factor = match &self.prefs.base {
            Some(_) => 1.0 / self.baseline(m, &cell.key)?,
            None => display(&self.data.unit(m)).factor,
        };
        let scaled = stat.scaled(factor);
        let spread = spread(&scaled, self.data.samples(m), self.unc);
        Some(Value {
            y: scaled.median,
            spread,
            stat: stat.clone(),
            invalid,
        })
    }

    fn ghost_value(&self, m: &MetricRef, cell: &CellInfo) -> Option<f64> {
        let stat = self.data.ghost(m, &cell.key)?;
        let factor = match &self.prefs.base {
            Some(_) => 1.0 / self.baseline(m, &cell.key)?,
            None => display(&self.data.unit(m)).factor,
        };
        Some(stat.median * factor)
    }

    /// Returns one series per framework and variant of a metric against the row count at a
    /// refresh rate, with direct end labels, plus ghost series of the compare run.
    pub fn rows_series(&self, scenario: &str, m: &MetricRef, hz: f64) -> Vec<Series> {
        self.series_by(scenario, m, hz, |k| Some(f64::from(k.rows)), |_| 1.0)
    }

    /// Returns one series per framework and variant of a metric at a refresh rate, with the x of
    /// a cell from `x_of` and its values times `per`, plus ghost series of the compare run.
    pub fn series_by(
        &self,
        scenario: &str,
        m: &MetricRef,
        hz: f64,
        x_of: impl Fn(&CellKey) -> Option<f64>,
        per: impl Fn(&CellKey) -> f64,
    ) -> Vec<Series> {
        let cells: Vec<&CellInfo> = self
            .cells(scenario)
            .into_iter()
            .filter(|c| c.key.hz == hz)
            .collect();
        let mut keys: Vec<(String, String)> = Vec::new();
        for c in &cells {
            let k = (c.key.framework.clone(), c.key.variant.clone());
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        let n = keys.len() as f64;
        let mut out = Vec::new();
        let mut ghosts = Vec::new();
        for (i, (fw, variant)) in keys.iter().enumerate() {
            let label = if variant == "default" {
                fw.clone()
            } else {
                format!("{fw} {variant}")
            };
            let mut s = Series::of_framework(format!("{fw}/{variant}"), fw, label.clone());
            s.marker = Marker::of_variant(variant);
            s.hollow = variant == "coarse";
            s.end_label = true;
            s.dodge = (i as f64 - (n - 1.0) / 2.0) * 2.2;
            let mut g = Series::of_framework(
                format!("ghost/{fw}/{variant}"),
                fw,
                format!("{label} (compare)"),
            );
            g.ghost = true;
            g.marker = s.marker;
            g.dodge = s.dodge;
            for c in cells
                .iter()
                .filter(|c| c.key.framework == *fw && c.key.variant == *variant)
            {
                let Some(x) = x_of(&c.key) else {
                    continue;
                };
                let k = per(&c.key);
                let times = |r: Option<(f64, f64)>| r.map(|(a, b)| (a * k, b * k));
                if let Some(v) = self.value(m, c) {
                    let mut p = Pt::new(x, v.y * k);
                    p.band = times(v.spread.band);
                    p.whisker = times(v.spread.whisker);
                    p.cell = Some(c.id);
                    p.invalid = v.invalid;
                    p.tip = cell_tip(c, &v.stat, self.data.samples(m));
                    s.points.push(p);
                }
                if let Some(y) = self.ghost_value(m, c) {
                    g.points.push(Pt::new(x, y * k));
                }
            }
            s.points.sort_by(|a, b| a.x.total_cmp(&b.x));
            g.points.sort_by(|a, b| a.x.total_cmp(&b.x));
            if !g.points.is_empty() {
                ghosts.push(g);
            }
            if !s.points.is_empty() {
                out.push(s);
            }
        }
        ghosts.extend(out);
        ghosts
    }

    /// Returns one dot row per shown cell of a scenario that passes a filter.
    pub fn dot_rows(
        &self,
        scenario: &str,
        m: &MetricRef,
        keep: impl Fn(&CellKey) -> bool,
    ) -> Vec<DotRow> {
        let cells: Vec<&CellInfo> = self
            .cells(scenario)
            .into_iter()
            .filter(|c| keep(&c.key))
            .collect();
        let many_rates = {
            let mut rates: Vec<f64> = cells.iter().map(|c| c.key.hz).collect();
            rates.dedup();
            rates.len() > 1
        };
        cells
            .iter()
            .map(|c| {
                let mut label = c.key.label();
                if many_rates {
                    label.push_str(&format!(" {} Hz", c.key.hz));
                }
                let mut row = DotRow::of_framework(c.id.to_string(), &c.key.framework, label);
                row.marker = Marker::of_variant(&c.key.variant);
                row.hollow = c.key.variant == "coarse";
                row.cell = Some(c.id);
                if let Some(v) = self.value(m, c) {
                    row.value = Some(v.y);
                    row.band = v.spread.band;
                    row.whisker = v.spread.whisker;
                    row.invalid = v.invalid;
                    row.text = if self.prefs.base.is_some() {
                        format!("{}×", short(v.y))
                    } else {
                        short(v.y)
                    };
                    row.tip = cell_tip(c, &v.stat, self.data.samples(m));
                }
                row.ghost = self.ghost_value(m, c);
                row
            })
            .collect()
    }
}
