//! One matrix cell: a fresh session at the cell's refresh rate, the app with the cell's scenario
//! and size, and the measurements of that scenario. Every cell writes `cell.json`, the app log,
//! screenshots and one JSON report per measurement into its own directory. A failed measurement
//! records its error and the cell goes on. A full cell launches the app several times, each in
//! its own session, for the startup times; the measurements run in the last one.

use std::cell::Cell as Flag;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use fakeland::{
    App, CaptureOptions, CaptureTarget, Fakeland, LaunchSpec, OutputConfig, SamplingConfig,
    Session, WindowRef,
};
use fakeland_bench::AnalysisInput;
use fakeland_bench::Analyzer;
use fakeland_bench::analyzers::Startup;
use fakeland_bench::analyzers::startup::StartupReport;
use fakeland_types::{Hz, Nanos};
use serde::Serialize;

use crate::dashboard::{self, OUTPUT};
use crate::frameworks::Framework;
use crate::scenarios::{Cell, Effort, Kind, Part};
use crate::steps;

/// The sample interval of resource sampling.
const SAMPLE_INTERVAL: Nanos = Nanos::from_millis(50);
/// How long an app may take after its first frames to show the layout of the spec.
const CONTENT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
/// The largest difference per color channel that the layout check accepts.
const LAYOUT_TOLERANCE: i32 = 6;

/// The outcome of one cell, as `cell.json` and the run index hold it.
#[derive(Debug, Clone, Serialize)]
pub struct CellOutcome {
    /// The cell.
    pub cell: Cell,
    /// The cell directory relative to the run directory.
    pub dir: String,
    /// The app arguments.
    pub args: Vec<String>,
    /// The measurements that wrote a report.
    pub done: Vec<String>,
    /// The measurements that failed, with their errors.
    pub errors: Vec<(String, String)>,
    /// The measurements the cell skips, with the reason.
    pub skipped: Vec<(String, String)>,
    /// A pacing measurement missed more than 1 % of the refreshes.
    pub missed_refreshes: bool,
    /// A response took over a second or never came, so the cell skipped its later measurements.
    pub over_budget: bool,
    /// Why the cell did not run: its framework was over the response budget at a smaller size.
    pub stopped: Option<String>,
}

impl CellOutcome {
    /// Returns the outcome of a cell before it runs.
    pub fn new(cell: &Cell) -> Self {
        Self {
            cell: cell.clone(),
            dir: cell.dir(),
            args: cell.app_args(),
            done: Vec::new(),
            errors: Vec::new(),
            skipped: Vec::new(),
            missed_refreshes: false,
            over_budget: false,
            stopped: None,
        }
    }
}

/// A running cell: its session, its app and where its files go.
pub struct CellRun<'a> {
    pub fw: &'a Framework,
    pub cell: Cell,
    pub effort: Effort,
    pub session: Session,
    pub app: App,
    pub out: PathBuf,
    pub outcome: CellOutcome,
    /// Set by a pacing measurement that misses refreshes.
    pub missed: Flag<bool>,
    /// Set by a probe whose response takes over a second or never comes.
    pub over: Flag<bool>,
}

impl CellRun<'_> {
    /// Runs one measurement and records whether it wrote its report.
    pub fn step(&mut self, name: &str, f: impl FnOnce(&CellRun) -> anyhow::Result<()>) {
        if self.over.get() {
            self.skip(name, "a response before took over a second or never came");
            return;
        }
        eprintln!("    {name}");
        match f(self) {
            Ok(()) => self.outcome.done.push(name.to_owned()),
            Err(e) => {
                // A wait that times out means the app no longer answers in time.
                if format!("{e:#}").contains("timed out") {
                    self.over.set(true);
                }
                eprintln!("    {name} failed: {e:#}");
                self.outcome
                    .errors
                    .push((name.to_owned(), format!("{e:#}")));
            }
        }
    }

    /// Records a measurement that the cell skips.
    pub fn skip(&mut self, name: &str, reason: &str) {
        self.outcome
            .skipped
            .push((name.to_owned(), reason.to_owned()));
    }

    /// Writes a value as pretty JSON into the cell directory.
    pub fn save(&self, file: &str, value: &impl Serialize) -> anyhow::Result<()> {
        save(&self.out.join(file), value)
    }
}

/// Runs one cell and writes its files below `run_dir`.
pub fn run(
    fw: &Framework,
    binary: &Path,
    cell: Cell,
    effort: Effort,
    a11y: bool,
    run_dir: &Path,
) -> CellOutcome {
    let mut outcome = CellOutcome::new(&cell);
    let out = run_dir.join(&outcome.dir);
    if let Err(e) = std::fs::create_dir_all(&out) {
        outcome.errors.push(("setup".into(), e.to_string()));
        return outcome;
    }
    let launches = match cell.part {
        Part::Full => effort.launches.max(1),
        Part::Pacing => 1,
    };
    let mut startups = Vec::new();
    let mut started = None;
    for launch in 0..launches {
        // Only the last session stays open; the earlier ones end before the next app starts.
        drop(started.take());
        match start(fw, binary, &cell, &outcome.args, a11y) {
            Ok((session, app, startup)) => {
                startups.push(startup);
                started = Some((session, app));
            }
            Err(e) => {
                outcome
                    .errors
                    .push(("startup".into(), format!("launch {launch}: {e:#}")));
                break;
            }
        }
    }
    let _ = save(
        &out.join("startup.json"),
        &serde_json::json!({ "launches": startups }),
    );
    let Some((session, app)) = started else {
        let _ = save(&out.join("cell.json"), &outcome);
        return outcome;
    };
    outcome.done.push("startup".into());
    let mut run = CellRun {
        fw,
        cell: cell.clone(),
        effort,
        session,
        app,
        out,
        outcome,
        missed: Flag::new(false),
        over: Flag::new(false),
    };
    match check_layout(&run) {
        Ok(()) => {
            run.outcome.done.push("layout".into());
            cell.kind.measure(&mut run);
        }
        Err(e) => {
            eprintln!("    layout check failed: {e:#}");
            run.outcome.errors.push(("layout".into(), format!("{e:#}")));
        }
    }
    run.outcome.missed_refreshes = run.missed.get();
    run.outcome.over_budget = run.over.get();
    let _ = std::fs::write(run.out.join("app.log"), run.app.stderr());
    if let Err(e) = run.session.blocking_shutdown() {
        run.outcome.errors.push(("shutdown".into(), e.to_string()));
    }
    let _ = run.save("cell.json", &run.outcome);
    run.outcome
}

/// Starts a session, launches the app, waits until it settles and returns the startup report.
fn start(
    fw: &Framework,
    binary: &Path,
    cell: &Cell,
    args: &[String],
    a11y: bool,
) -> anyhow::Result<(Session, App, serde_json::Value)> {
    let session = Fakeland::builder()
        .output(OutputConfig::new("DP-1").mode(OUTPUT.0, OUTPUT.1, Hz(cell.hz)))
        .app_marks(fw.marks)
        .accessibility(a11y)
        .sampling(SamplingConfig::every(SAMPLE_INTERVAL))
        .start()
        .context("start the session")?;
    let app = session
        .blocking_launch(LaunchSpec::new(binary.to_string_lossy()).args(args.iter().cloned()))
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
        .with_context(|| format!("the window never settles: {}", app.stderr()))?;
    // Some apps commit an empty frame first and draw their content later. The startup ends
    // with the frame that shows the layout of the spec.
    let waited = std::time::Instant::now();
    while !wrong_points(&session, cell.kind)?.is_empty() && waited.elapsed() < CONTENT_TIMEOUT {
        let _ = session.blocking_wait_frames(WindowRef::id(window.id), 1, Nanos::from_secs(2));
        let _ = session.blocking_wait_settled(
            WindowRef::id(window.id),
            Nanos::from_millis(300),
            Nanos::from_secs(60),
        );
    }
    let input = AnalysisInput::new(session.blocking_trace_snapshot()?);
    let startup = Startup {
        command: Some(fw.binary()),
    }
    .analyze(&input);
    let mut report = serde_json::to_value(&startup)?;
    report["content_ms"] = serde_json::json!(content_ms(&input, &startup));
    Ok((session, app, report))
}

/// Returns the time from the spawn of the app to the presentation of the last frame of its
/// window in the trace, in milliseconds. After the window settles with the layout of the spec,
/// that frame is the first one that shows the whole content.
fn content_ms(input: &AnalysisInput, startup: &StartupReport) -> Option<f64> {
    let (spawned, window) = (startup.spawned?, startup.window?);
    let timeline = &input.timeline;
    timeline
        .window_surfaces(window)
        .iter()
        .flat_map(|surface| timeline.frames(*surface).iter())
        .filter(|frame| frame.is_presented_frame())
        .filter_map(|frame| frame.presented_time())
        .max()
        .map(|t| t.since(spawned).as_millis_f64())
}

/// Returns the points of the layout check whose color differs from the spec, as text.
fn wrong_points(session: &Session, kind: Kind) -> anyhow::Result<Vec<String>> {
    let shot = session.blocking_screenshot(
        CaptureTarget::default(),
        CaptureOptions {
            include_cursor: false,
            max_dim: None,
        },
    )?;
    Ok(wrong_in(&shot.image, kind))
}

/// Returns the points of an image whose color differs from the spec, as text.
fn wrong_in(image: &fakeland::Image, kind: Kind) -> Vec<String> {
    let mut wrong = Vec::new();
    for &(x, y, want) in dashboard::checks(kind) {
        let got = image.pixel(x as u32, y as u32).unwrap_or([0; 4]);
        let want = [(want >> 16) as u8, (want >> 8) as u8, want as u8];
        let off = (0..3).any(|i| (i32::from(got[i]) - i32::from(want[i])).abs() > LAYOUT_TOLERANCE);
        if off {
            wrong.push(format!(
                "({x}, {y}) is #{:02x}{:02x}{:02x}, not #{:02x}{:02x}{:02x}",
                got[0], got[1], got[2], want[0], want[1], want[2]
            ));
        }
    }
    wrong
}

/// Takes a screenshot into `screenshot.png` and checks the colors at the points the layout of the
/// scenario fixes. A mismatch means the app draws another layout, and its probes would watch the
/// wrong pixels.
fn check_layout(run: &CellRun) -> anyhow::Result<()> {
    // The pointer starts at the top left corner. It moves where no hover color reaches the
    // points the check reads.
    steps::park(run, dashboard::PARK)?;
    let shot = steps::screenshot(run, "screenshot")?;
    let wrong = wrong_in(&shot.image, run.cell.kind);
    if !wrong.is_empty() {
        bail!("the layout differs from the spec: {}", wrong.join("; "));
    }
    Ok(())
}

/// Writes a value as pretty JSON.
pub fn save(path: &PathBuf, value: &impl Serialize) -> anyhow::Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
}
