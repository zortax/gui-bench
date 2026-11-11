//! One matrix cell: a fresh session at the cell's refresh rate, the app with the cell's row count,
//! and the startup, hover and scroll scenarios. Each scenario writes its report as JSON. A failed
//! scenario records its error and the cell goes on.

use std::path::{Path, PathBuf};

use anyhow::Context;
use fakeland::{Fakeland, LaunchSpec, OutputConfig, Session, WindowRef};
use fakeland_bench::analyzers::Startup;
use fakeland_bench::presets::{self, PresetParams};
use fakeland_bench::{AnalysisInput, Analyzer, BenchRun};
use fakeland_types::{Hz, Nanos};
use serde::Serialize;

use crate::frameworks::Framework;
use crate::matrix::{Cell, Effort, OUTPUT};
use crate::phases;

/// The outcome of one cell.
#[derive(Debug, Serialize)]
pub struct CellOutcome {
    /// The framework name.
    pub framework: String,
    /// The cell.
    pub cell: Cell,
    /// The cell directory relative to the run directory.
    pub dir: String,
    /// The scenarios that wrote a report.
    pub done: Vec<String>,
    /// The scenarios that failed, with their errors.
    pub errors: Vec<(String, String)>,
    /// The scenarios the cell skips, with the reason.
    pub skipped: Vec<(String, String)>,
}

/// Runs one cell and writes its files below `run_dir`.
pub fn run(
    fw: &Framework,
    binary: &Path,
    cell: Cell,
    effort: Effort,
    run_dir: &Path,
) -> CellOutcome {
    let dir = format!("{}/{}", fw.name, cell.dir());
    let mut outcome = CellOutcome {
        framework: fw.name.to_owned(),
        cell,
        dir: dir.clone(),
        done: Vec::new(),
        errors: Vec::new(),
        skipped: Vec::new(),
    };
    let out = run_dir.join(&dir);
    if let Err(e) = std::fs::create_dir_all(&out) {
        outcome.errors.push(("setup".into(), e.to_string()));
        return outcome;
    }
    let session = match start(fw, binary, cell, &out) {
        Ok(session) => session,
        Err(e) => {
            outcome.errors.push(("startup".into(), format!("{e:#}")));
            return outcome;
        }
    };
    outcome.done.push("startup".into());

    let mut step = |name: &str, f: &mut dyn FnMut() -> anyhow::Result<()>| match f() {
        Ok(()) => outcome.done.push(name.to_owned()),
        Err(e) => {
            eprintln!("    {name} failed: {e:#}");
            outcome.errors.push((name.to_owned(), format!("{e:#}")));
        }
    };
    step("hover", &mut || {
        let run = bench(
            &session,
            "hover-latency",
            &cell.hover_params("pixels", effort),
        )?;
        save(&out.join("hover.json"), &run.report())?;
        if fw.marks {
            save(&out.join("phases-hover.json"), &phases_of(&run))?;
        }
        Ok(())
    });
    if fw.marks {
        step("hover-marks", &mut || {
            let run = bench(
                &session,
                "hover-latency",
                &cell.hover_params("app_mark", effort),
            )?;
            save(&out.join("hover-marks.json"), &run.report())
        });
    }
    if cell.scrolls() {
        step("scroll", &mut || {
            let run = bench(&session, "scroll-glide", &cell.scroll_params(effort))?;
            save(&out.join("scroll.json"), &run.report())?;
            if fw.marks {
                save(&out.join("phases-scroll.json"), &phases_of(&run))?;
            }
            Ok(())
        });
    } else {
        outcome
            .skipped
            .push(("scroll".into(), "the list is shorter than a glide".into()));
    }
    if let Err(e) = session.blocking_shutdown() {
        outcome.errors.push(("shutdown".into(), e.to_string()));
    }
    outcome
}

/// Starts a session, launches the app, waits until it settles and writes the startup report.
fn start(fw: &Framework, binary: &Path, cell: Cell, out: &Path) -> anyhow::Result<Session> {
    let session = Fakeland::builder()
        .output(OutputConfig::new("DP-1").mode(OUTPUT.0, OUTPUT.1, Hz(cell.hz)))
        .app_marks(fw.marks)
        .start()
        .context("start the session")?;
    let rows = cell.rows.to_string();
    let app = session
        .blocking_launch(LaunchSpec::new(binary.to_string_lossy()).args(["--rows", &rows]))
        .context("launch the app")?;
    let window = app
        .blocking_wait_window(Nanos::from_secs(120))
        .with_context(|| format!("no window: {}", app.stderr()))?;
    session
        .blocking_wait_settled(
            WindowRef::id(window.id),
            Nanos::from_millis(300),
            Nanos::from_secs(60),
        )
        .context("the window never settles")?;
    let trace = session.blocking_trace_snapshot()?;
    let startup = Startup {
        command: Some(fw.binary.to_owned()),
    }
    .analyze(&AnalysisInput::new(trace));
    save(&out.join("startup.json"), &startup)?;
    Ok(session)
}

fn bench(session: &Session, preset: &str, params: &PresetParams) -> anyhow::Result<BenchRun> {
    let bench = presets::by_name(preset)
        .with_context(|| format!("no preset {preset}"))?
        .bench(params)?;
    Ok(bench.blocking_run(session)?)
}

fn phases_of(run: &BenchRun) -> impl Serialize {
    phases::collect(
        run.measured()
            .filter_map(|it| it.input.as_ref().map(|input| &input.trace)),
    )
}

/// Writes a value as pretty JSON.
pub fn save(path: &PathBuf, value: &impl Serialize) -> anyhow::Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
}
