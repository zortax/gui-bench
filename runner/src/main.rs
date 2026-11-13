//! Runs the GUI benchmark matrix under fakeland. For every scenario, framework and size, the
//! runner starts a headless session at 144 Hz, launches the framework's app and runs the
//! scenario's measurements. Results go to `results/<run>/<scenario>/<framework>/<hz>hz/<size>/`
//! as JSON, with `run.json` as the index; `results/latest` links to the newest run.
//!
//! The sizes of a framework run smallest first. Once its hover response passes the budget, or
//! any response takes over a second, its larger sizes do not run and count as over budget. A cell that misses refreshes while it
//! scrolls, resizes or animates runs those measurements again at 60 Hz. `--resume` continues a
//! run, or grows it: cells outside the new matrix stay in the index.

mod cell;
mod dashboard;
mod frameworks;
mod index;
mod limits;
mod phases;
mod scenarios;
mod steps;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail};
use clap::Parser;
use serde_json::Value;

use crate::cell::CellOutcome;
use crate::frameworks::Framework;
use crate::index::{Index, Invocation};
use crate::scenarios::{Cell, Effort, Kind, MAIN_HZ, Part, SLOW_HZ};

/// The median hover response in milliseconds above which a framework stops at a size.
const BUDGET_MS: f64 = 250.0;

/// Runs the GUI benchmark matrix under fakeland.
#[derive(Debug, Parser)]
struct Args {
    /// The scenarios, comma separated: app, list.
    #[arg(long, value_delimiter = ',', default_value = "app,list")]
    scenarios: Vec<String>,
    /// The frameworks, comma separated. The default leaves out the editions that compare
    /// versions or renderers of a framework: gpui-kit-fast, freya-056a5b4, dioxus-skia.
    #[arg(long, value_delimiter = ',')]
    frameworks: Vec<String>,
    /// The row counts, comma separated, for every selected scenario. Each scenario has its own
    /// default.
    #[arg(long, value_delimiter = ',')]
    rows: Vec<u32>,
    /// The refresh rate of the output in Hz.
    #[arg(long, default_value_t = MAIN_HZ)]
    hz: f64,
    /// Hover repeats per iteration.
    #[arg(long, default_value_t = 10)]
    repeats: u32,
    /// Repeats per iteration of the click and key probes.
    #[arg(long, default_value_t = 6)]
    op_repeats: u32,
    /// Measured iterations of every measurement.
    #[arg(long, default_value_t = 3)]
    iterations: u32,
    /// App launches of the startup measurement.
    #[arg(long, default_value_t = 3)]
    launches: u32,
    /// How often a measurement runs again when the host disturbs it. On a host that
    /// `--allow-noisy-host` lets pass, measurements do not run again.
    #[arg(long, default_value_t = 2)]
    retries: u32,
    /// A small matrix for a quick check: two sizes of the app, one of the list, fewer repeats.
    #[arg(long)]
    quick: bool,
    /// The whole matrix for a publication: every edition, five iterations, and every size even
    /// when a framework is over the budget.
    #[arg(long, conflicts_with = "quick")]
    full: bool,
    /// Run on a host that disturbs the timing, such as one on the `powersave` governor.
    #[arg(long)]
    allow_noisy_host: bool,
    /// Continue a run in `--out`: cells whose `cell.json` has no errors keep their results, the
    /// other cells run again. Cells of the earlier `run.json` outside the selected matrix stay in
    /// the index, so a run can grow.
    #[arg(long, requires = "out")]
    resume: bool,
    /// Run an accessibility bus in every session, so apps publish their accessibility trees as
    /// they do while an assistive technology runs. Off by default, as on a desktop without one.
    #[arg(long)]
    a11y: bool,
    /// Leave `results/latest` as it is.
    #[arg(long)]
    keep_latest: bool,
    /// The run directory. The default is `results/run-<unix time>`.
    #[arg(long)]
    out: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    limits::raise_open_files();
    if check_host(args.allow_noisy_host)? {
        args.retries = 0;
    }
    if args.quick {
        args.repeats = 6;
        args.op_repeats = 4;
        args.iterations = 2;
        args.launches = 2;
    }
    if args.full {
        args.iterations = 5;
        args.launches = 5;
    }
    if args.frameworks.is_empty() {
        args.frameworks = if args.full {
            frameworks::ALL.iter().map(|f| f.name.to_owned()).collect()
        } else {
            Framework::defaults()
        };
    }
    let root = frameworks::root();
    let mut kinds = Vec::new();
    for name in &args.scenarios {
        kinds.push(Kind::by_name(name).with_context(|| format!("unknown scenario {name}"))?);
    }
    let mut selected = Vec::new();
    for name in &args.frameworks {
        let fw = Framework::by_name(name).with_context(|| format!("unknown framework {name}"))?;
        let binary = fw.binary_path(&root);
        if !binary.exists() {
            bail!("{} is missing; run `just build`", binary.display());
        }
        selected.push((fw, binary));
    }
    let effort = Effort {
        hover_repeats: args.repeats,
        op_repeats: args.op_repeats,
        iterations: args.iterations,
        launches: args.launches,
        retries: args.retries,
    };
    let cells = matrix(&args, &kinds, &selected);
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let run_dir = args
        .out
        .clone()
        .unwrap_or_else(|| root.join("results").join(format!("run-{stamp}")));
    std::fs::create_dir_all(&run_dir)?;
    if !args.keep_latest {
        link_latest(&root, &run_dir);
    }

    let invocation = Invocation {
        scenarios: &args.scenarios,
        frameworks: &args.frameworks,
        effort,
        started_unix: stamp,
        a11y: args.a11y,
    };
    let run_index = Index::new(&run_dir, &invocation, args.resume);
    let mut run = Run {
        args: &args,
        effort,
        run_dir: &run_dir,
        index: run_index,
        outcomes: Vec::new(),
        stopped: HashSet::new(),
    };
    let total = cells.len();
    for (at, (fw, binary, cell)) in cells.into_iter().enumerate() {
        eprintln!("[{}/{total}] {}", at + 1, cell.dir());
        run.cell(fw, binary, cell)?;
    }
    let failed: usize = run
        .outcomes
        .iter()
        .filter_map(|c| c["errors"].as_array().map(Vec::len))
        .sum();
    eprintln!(
        "results: {} ({failed} failed measurements)",
        run_dir.display()
    );
    Ok(())
}

/// A run of the matrix: its settings, its index, the outcomes so far, and the frameworks that
/// are over the budget in a scenario.
struct Run<'a> {
    args: &'a Args,
    effort: Effort,
    run_dir: &'a Path,
    index: Index,
    outcomes: Vec<Value>,
    stopped: HashSet<(Kind, String)>,
}

impl Run<'_> {
    /// Runs one cell of the matrix, and its pacing measurements again at 60 Hz when it misses
    /// refreshes. A cell of a framework that is over the budget is recorded and not run.
    fn cell(&mut self, fw: &Framework, binary: &Path, cell: Cell) -> anyhow::Result<()> {
        let key = (cell.kind, cell.framework.clone());
        if self.stopped.contains(&key) {
            eprintln!("    not run: over the response budget at a smaller size");
            let mut outcome = CellOutcome::new(&cell);
            outcome.stopped = Some("over the response budget at a smaller size".to_owned());
            let dir = self.run_dir.join(cell.dir());
            std::fs::create_dir_all(&dir)?;
            cell::save(&dir.join("cell.json"), &outcome)?;
            return self.record(serde_json::to_value(outcome)?);
        }
        let outcome = self.run_or_keep(fw, binary, &cell)?;
        let over = outcome["over_budget"].as_bool().unwrap_or(false);
        if !self.args.full && (over || over_budget(&self.run_dir.join(cell.dir()))) {
            eprintln!("    over the response budget: larger sizes do not run");
            self.stopped.insert(key);
        }
        let missed = outcome["missed_refreshes"].as_bool().unwrap_or(false);
        self.record(outcome)?;
        if missed && cell.hz > SLOW_HZ {
            let slow = Cell::new(cell.kind, &cell.framework, cell.rows, SLOW_HZ, Part::Pacing);
            eprintln!("    missed refreshes: {}", slow.dir());
            let outcome = self.run_or_keep(fw, binary, &slow)?;
            self.record(outcome)?;
        }
        Ok(())
    }

    /// Runs a cell, or with `--resume` keeps the outcome of an earlier run without errors.
    fn run_or_keep(&self, fw: &Framework, binary: &Path, cell: &Cell) -> anyhow::Result<Value> {
        let dir = self.run_dir.join(cell.dir());
        if self.args.resume {
            if let Some(done) = finished(&dir) {
                eprintln!("    kept");
                return Ok(done);
            }
            let _ = std::fs::remove_dir_all(&dir);
        }
        let outcome = cell::run(
            fw,
            binary,
            cell.clone(),
            self.effort,
            self.args.a11y,
            self.run_dir,
        );
        Ok(serde_json::to_value(outcome)?)
    }

    /// Adds an outcome and writes the index.
    fn record(&mut self, outcome: Value) -> anyhow::Result<()> {
        self.outcomes.push(outcome);
        self.index.write(&self.outcomes)
    }
}

/// Returns true when the hover response of a cell is over the budget: its median is above
/// `BUDGET_MS`, a probe timed out, or the measurement left no report.
fn over_budget(dir: &Path) -> bool {
    let Some(report) = std::fs::read_to_string(dir.join("hover-row.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
    else {
        return dir.join("cell.json").exists() && !dir.join("screenshot.png").exists();
    };
    let slow = steps::metric(&report, "latency", "total_p50").is_none_or(|ms| ms > BUDGET_MS);
    let timeouts = steps::metric(&report, "latency", "timeouts").is_some_and(|n| n > 0.0);
    slow || timeouts
}

/// Stops on a host whose settings disturb the timing, unless `allow` is set. Returns true for
/// such a host.
fn check_host(allow: bool) -> anyhow::Result<bool> {
    let governor = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .unwrap_or_default();
    if governor.trim() != "powersave" {
        return Ok(false);
    }
    let advice = "the CPU governor is `powersave`, which makes the compositor wake late and \
                  the reports invalid; set `performance` (for example `sudo cpupower \
                  frequency-set -g performance`)";
    if allow {
        eprintln!("warning: {advice}");
        return Ok(true);
    }
    bail!("{advice}, or pass --allow-noisy-host");
}

/// Returns every cell of the selected scenarios and frameworks, in run order: scenario, then
/// framework, then size, smallest first.
fn matrix<'a>(
    args: &Args,
    kinds: &[Kind],
    selected: &'a [(&'static Framework, PathBuf)],
) -> Vec<(&'static Framework, &'a Path, Cell)> {
    let mut cells = Vec::new();
    for &kind in kinds {
        let mut rows = if args.rows.is_empty() {
            kind.row_counts(args.quick)
        } else {
            args.rows.clone()
        };
        rows.sort_unstable();
        for (fw, binary) in selected {
            for &count in &rows {
                let cell = Cell::new(kind, fw.name, count, args.hz, Part::Full);
                cells.push((*fw, binary.as_path(), cell));
            }
        }
    }
    cells
}

/// Returns the outcome of a cell that a previous run finished without errors, or `None`.
fn finished(dir: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(dir.join("cell.json")).ok()?;
    let outcome: serde_json::Value = serde_json::from_str(&text).ok()?;
    let clean = outcome["errors"].as_array().is_some_and(Vec::is_empty);
    clean.then_some(outcome)
}

/// Points `results/latest` at the run directory.
fn link_latest(root: &Path, run_dir: &Path) {
    let latest = root.join("results/latest");
    let _ = std::fs::remove_file(&latest);
    let target = run_dir
        .canonicalize()
        .unwrap_or_else(|_| run_dir.to_path_buf());
    if let Err(e) = std::os::unix::fs::symlink(&target, &latest) {
        eprintln!("cannot link {}: {e}", latest.display());
    }
}
