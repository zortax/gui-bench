//! The cells that the preferences let through, in plot order, and the axes of a scenario: its
//! frameworks, rates and sizes.

use explorer_core::api::{CellInfo, RunOverview};

use super::PrefsSnapshot;
use crate::palette;

/// Returns the shown cells of a scenario that ran, in plot order: framework, rate, size.
pub fn cells_of<'a>(
    overview: &'a RunOverview,
    prefs: &PrefsSnapshot,
    scenario: &str,
) -> Vec<&'a CellInfo> {
    let mut cells: Vec<&CellInfo> = overview
        .cells
        .iter()
        .filter(|c| c.key.scenario == scenario && c.ran())
        .filter(|c| prefs.shows_framework(&c.key.framework))
        .filter(|c| prefs.shows_rate(c.key.hz))
        .collect();
    cells.sort_by(|a, b| {
        palette::rank(&a.key.framework)
            .total_cmp(&palette::rank(&b.key.framework))
            .then_with(|| a.key.framework.cmp(&b.key.framework))
            .then_with(|| a.key.hz.total_cmp(&b.key.hz))
            .then_with(|| a.key.rows.cmp(&b.key.rows))
    });
    cells
}

/// Returns the smallest size of a scenario that a framework did not run, since it was over the
/// response budget at a smaller size.
pub fn stop_of(overview: &RunOverview, scenario: &str, framework: &str) -> Option<f64> {
    overview
        .cells
        .iter()
        .filter(|c| c.key.scenario == scenario && c.key.framework == framework && !c.ran())
        .map(|c| c.key.size())
        .min_by(f64::total_cmp)
}

/// A size of a scenario: the rows, and the elements of the app.
pub type Size = (u32, Option<u32>);

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

    /// Returns the sizes in ascending order, of the cells at a refresh rate or of every cell.
    pub fn sizes(&self, hz: Option<f64>) -> Vec<Size> {
        let mut out: Vec<Size> = Vec::new();
        for c in self
            .cells
            .iter()
            .filter(|c| hz.is_none_or(|h| c.key.hz == h))
        {
            let s = (c.key.rows, c.key.elements);
            if !out.contains(&s) {
                out.push(s);
            }
        }
        out.sort_by_key(|(r, _)| *r);
        out
    }

    /// Returns the number of cells at a refresh rate and a row count.
    pub fn count(&self, hz: f64, rows: u32) -> usize {
        self.cells
            .iter()
            .filter(|c| c.key.hz == hz && c.key.rows == rows)
            .count()
    }
}
