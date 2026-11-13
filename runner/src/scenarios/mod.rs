//! The scenarios: what each one asks the app to show, at which sizes it runs and what it
//! measures. `Cell` names one point of the matrix and its directory in the run.

mod measure;

use serde::Serialize;

/// The refresh rate of every cell.
pub const MAIN_HZ: f64 = 144.0;
/// The refresh rate of the second pass of the pacing steps, for cells that miss refreshes at
/// `MAIN_HZ`.
pub const SLOW_HZ: f64 = 60.0;

/// One scenario of the matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// The workbench app of `apps/shared/dashboard.rs` with N orders and N messages, all real
    /// elements: responses to hover, clicks and keys, scrolling, resizing, live updates and page
    /// switches.
    App,
    /// The order rows in the framework's virtual list, with N rows of data.
    List,
}

impl Kind {
    /// Every scenario in run order.
    pub const ALL: [Kind; 2] = [Kind::App, Kind::List];

    /// Returns the name in results, on the command line and for the app.
    pub fn name(self) -> &'static str {
        match self {
            Kind::App => "app",
            Kind::List => "list",
        }
    }

    /// Returns the scenario with a name.
    pub fn by_name(name: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.name() == name)
    }

    /// Returns the default row counts, smallest first.
    pub fn row_counts(self, quick: bool) -> Vec<u32> {
        match (self, quick) {
            (Kind::App, false) => vec![30, 300, 1000, 3000],
            (Kind::App, true) => vec![30, 1000],
            (Kind::List, false) => vec![10_000, 1_000_000],
            (Kind::List, true) => vec![10_000],
        }
    }

    /// Returns the number of elements the spec counts in the app at a row count, which results
    /// use as the size of the app. A list holds only the rows of its window.
    pub fn elements(self, rows: u32) -> Option<u32> {
        match self {
            Kind::App => {
                Some(crate::dashboard::CHROME_ELEMENTS + crate::dashboard::ORDER_ELEMENTS * rows)
            }
            Kind::List => None,
        }
    }

    /// Runs the measurements of a cell.
    pub fn measure(self, run: &mut crate::cell::CellRun) {
        measure::run(self, run);
    }
}

/// Which measurements a cell takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Part {
    /// Every measurement of the scenario.
    Full,
    /// The pacing measurements only: scrolling, resizing and live updates. It runs at `SLOW_HZ`
    /// for a cell that misses refreshes at `MAIN_HZ`.
    Pacing,
}

/// The repeat counts of the measurements.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Effort {
    /// Hover repeats per iteration.
    pub hover_repeats: u32,
    /// Repeats per iteration of the click and key probes.
    pub op_repeats: u32,
    /// Measured iterations of every measurement.
    pub iterations: u32,
    /// App launches of the startup measurement.
    pub launches: u32,
    /// How often a measurement runs again when the host disturbs it.
    pub retries: u32,
}

/// One cell of the matrix.
#[derive(Debug, Clone, Serialize)]
pub struct Cell {
    /// The scenario.
    #[serde(rename = "scenario")]
    pub kind: Kind,
    /// The framework name.
    pub framework: String,
    /// The refresh rate of the output in Hz.
    pub hz: f64,
    /// The number of rows the app holds.
    pub rows: u32,
    /// The elements of the app as the spec counts them.
    pub elements: Option<u32>,
    /// The measurements the cell takes.
    pub part: Part,
}

impl Cell {
    /// Returns a cell of a scenario, a framework and a size.
    pub fn new(kind: Kind, framework: &str, rows: u32, hz: f64, part: Part) -> Self {
        Self {
            kind,
            framework: framework.to_owned(),
            hz,
            rows,
            elements: kind.elements(rows),
            part,
        }
    }

    /// Returns the directory of the cell in the run: `<scenario>/<framework>/<hz>hz/<rows>`.
    pub fn dir(&self) -> String {
        format!(
            "{}/{}/{}hz/{}",
            self.kind.name(),
            self.framework,
            self.hz,
            self.rows
        )
    }

    /// Returns the arguments of the app.
    pub fn app_args(&self) -> Vec<String> {
        vec![
            "--scenario".to_owned(),
            self.kind.name().to_owned(),
            "--rows".to_owned(),
            self.rows.to_string(),
        ]
    }
}
