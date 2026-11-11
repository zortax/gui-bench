//! The cells that the preferences let through, in plot order, and the axes of a scenario: its
//! frameworks, rates and sizes.

use explorer_core::api::{CellInfo, RunOverview};

use super::PrefsSnapshot;
use crate::palette;

/// Returns the shown cells of a scenario in plot order: framework, variant, rate, size.
pub fn cells_of<'a>(
    overview: &'a RunOverview,
    prefs: &PrefsSnapshot,
    scenario: &str,
) -> Vec<&'a CellInfo> {
    let mut cells: Vec<&CellInfo> = overview
        .cells
        .iter()
        .filter(|c| c.key.scenario == scenario)
        .filter(|c| prefs.shows_framework(&c.key.framework))
        .filter(|c| prefs.shows_variant(&c.key.variant))
        .filter(|c| prefs.shows_rate(c.key.hz))
        .collect();
    cells.sort_by(|a, b| {
        palette::rank(&a.key.framework)
            .total_cmp(&palette::rank(&b.key.framework))
            .then_with(|| a.key.framework.cmp(&b.key.framework))
            .then_with(|| {
                palette::variant_rank(&a.key.variant).cmp(&palette::variant_rank(&b.key.variant))
            })
            .then_with(|| a.key.hz.total_cmp(&b.key.hz))
            .then_with(|| a.key.size_key().cmp(&b.key.size_key()))
    });
    cells
}

/// The shown cells of one scenario with their axes.
#[derive(Debug, Clone)]
pub struct Selection<'a> {
    pub cells: Vec<&'a CellInfo>,
}

impl<'a> Selection<'a> {
    /// Returns the shown cells of a scenario.
    pub fn new(overview: &'a RunOverview, prefs: &PrefsSnapshot, scenario: &str) -> Self {
        Self {
            cells: cells_of(overview, prefs, scenario),
        }
    }

    /// Returns the frameworks in plot order.
    pub fn frameworks(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for c in &self.cells {
            if !out.contains(&c.key.framework) {
                out.push(c.key.framework.clone());
            }
        }
        out
    }

    /// Returns the refresh rates in ascending order.
    pub fn rates(&self) -> Vec<f64> {
        let mut out: Vec<f64> = Vec::new();
        for c in &self.cells {
            if !out.contains(&c.key.hz) {
                out.push(c.key.hz);
            }
        }
        out.sort_by(f64::total_cmp);
        out
    }

    /// Returns the sizes (rows, updates) in ascending order.
    pub fn sizes(&self) -> Vec<(u32, Option<u32>)> {
        let mut out: Vec<(u32, Option<u32>)> = Vec::new();
        for c in &self.cells {
            let s = (c.key.rows, c.key.updates);
            if !out.contains(&s) {
                out.push(s);
            }
        }
        out.sort_by_key(|(r, u)| (*r, u.unwrap_or(0)));
        out
    }
}
