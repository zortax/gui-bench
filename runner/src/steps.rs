//! The measurements that scenarios share. Each one runs a fakeland bench preset with parameters
//! from the cell, adds the resource and frame-cost analyzers, and writes the report as
//! `<name>.json`. Apps with app marks also get `phases-<name>.json`.

use anyhow::Context;
use fakeland::scenario::prelude::*;
use fakeland_bench::analyzers::{FrameCost, Resources};
use fakeland_bench::presets::{self, PresetParams};
use fakeland_bench::{Bench, BenchRun};
use serde::Serialize;
use serde_json::json;

use crate::cell::CellRun;
use crate::dashboard::Target;
use crate::layout::{OUTPUT, Page, ROW_HEIGHT};
use crate::phases;

/// The scroll velocity in logical pixels per second.
pub const SCROLL_VELOCITY: f64 = 800.0;
/// The length of one scroll glide in seconds.
pub const SCROLL_SECONDS: f64 = 1.0;
/// The window y of the pointer during a glide.
const SCROLL_POINTER_Y: i32 = 720;
/// The height of the region that motion estimation reads, from the top of the window. It ends
/// six rows above the pointer, so the hover color that trails the pointer stays outside.
const SCROLL_ROI_HEIGHT: i32 = 512;
/// The row that the hover probe enters, and the row the pointer rests on between repeats.
const HOVER_ROW: i32 = 5;
const AWAY_ROW: i32 = 1;
/// How long the pacing and idle measurements last.
const WINDOW_SECONDS: &str = "3s";
/// The probe timeout: a coarse rebuild of 10 000 rows can take a while.
const PROBE_TIMEOUT: &str = "5s";

/// Builds a preset bench, adds the resource and frame-cost analyzers, and runs it.
pub fn bench(run: &CellRun, preset: &str, params: &PresetParams) -> anyhow::Result<BenchRun> {
    let bench = presets::by_name(preset)
        .with_context(|| format!("no preset {preset}"))?
        .bench(params)?;
    Ok(with_cost(bench).blocking_run(&run.session)?)
}

/// Adds the resource analyzer when the bench lacks it, and the frame-cost analyzer.
fn with_cost(bench: Bench) -> Bench {
    let bench = if bench.analyzer_names().iter().any(|n| n == "resources") {
        bench
    } else {
        bench.analyze(Resources)
    };
    bench.analyze(FrameCost::default())
}

/// Runs a preset bench and writes its report as `<name>.json`.
pub fn measure(
    run: &CellRun,
    name: &str,
    preset: &str,
    params: &PresetParams,
) -> anyhow::Result<()> {
    let result = bench(run, preset, params)?;
    run.save(&format!("{name}.json"), &result.report())?;
    if run.fw.marks {
        run.save(&format!("phases-{name}.json"), &phases_of(&result))?;
    }
    Ok(())
}

fn phases_of(run: &BenchRun) -> impl Serialize {
    phases::collect(
        run.measured()
            .filter_map(|it| it.input.as_ref().map(|input| &input.trace)),
    )
}

/// Returns the parameters every bench shares: the iterations and one warmup.
fn common(iterations: u32) -> PresetParams {
    PresetParams::new()
        .set("iterations", iterations)
        .set("warmup", 1)
}

/// The idle measurement: three seconds with no input, the middle third suspended, every commit
/// captured for the damage analysis.
pub fn idle() -> PresetParams {
    PresetParams::new()
        .set("iterations", 1)
        .set("duration", WINDOW_SECONDS)
        .set("suspend", true)
        .set("capture", true)
        .set("downscale", 2)
}

/// The pacing measurement of a running animation or ticker.
pub fn pacing(run: &CellRun) -> PresetParams {
    common(run.effort.iterations).set("duration", WINDOW_SECONDS)
}

/// The hover probe: the pointer enters row 5 from row 1, and the pixels of row 5 tell the change.
/// `detector` is `pixels` or `app_mark`.
pub fn hover(run: &CellRun, page: Page, detector: &str) -> PresetParams {
    let x = OUTPUT.0 / 2;
    let target = Target {
        at: (x, page.row_center(HOVER_ROW)),
        away: (x, page.row_center(AWAY_ROW)),
        roi: page.rows_roi(HOVER_ROW, 1),
    };
    hover_at(run, &target, detector)
}

/// A hover probe of any element: the pointer enters `target.at` from `target.away`, and the
/// pixels of `target.roi` tell the change.
pub fn hover_at(run: &CellRun, target: &Target, detector: &str) -> PresetParams {
    common(run.effort.iterations)
        .set("position", point(target.at))
        .set("away", point(target.away))
        .set("roi", target.roi.clone())
        .set("detector", detector)
        .set("repeats", run.effort.hover_repeats)
        .set("timeout", "2s")
}

/// Returns a window point as preset JSON.
fn point((x, y): (i32, i32)) -> serde_json::Value {
    json!({ "x": x, "y": y })
}

/// Returns true when a list of `rows` rows is tall enough for a glide in both directions.
pub fn scrolls(rows: u32) -> bool {
    let travel = SCROLL_VELOCITY * SCROLL_SECONDS;
    f64::from(rows) * f64::from(ROW_HEIGHT) >= f64::from(OUTPUT.1) + travel * 1.5
}

/// The scroll glide: a finger at 800 px/s with no kinetic tail, four events per refresh, so every
/// frame consumes the same scroll. Motion estimation reads the rows above the pointer.
pub fn scroll(run: &CellRun) -> PresetParams {
    let roi = json!({ "x": 0, "y": 0, "w": OUTPUT.0, "h": SCROLL_ROI_HEIGHT });
    scroll_at(run, (OUTPUT.0 / 2, SCROLL_POINTER_Y), roi)
}

/// The scroll glide with the pointer at `pointer`, and motion estimation in `roi`.
pub fn scroll_at(run: &CellRun, pointer: (i32, i32), roi: serde_json::Value) -> PresetParams {
    PresetParams::new()
        .set("source", json!({ "type": "finger", "kinetic": false }))
        .set("velocity", SCROLL_VELOCITY)
        .set("duration", format!("{SCROLL_SECONDS}s"))
        .set("rate_hz", run.cell.hz * 4.0)
        .set("position", point(pointer))
        .set("roi", roi)
        .set("iterations", run.effort.iterations + 1)
        .set("warmup", 1)
}

/// A click probe on a row: the first click selects it, the second clears the selection. The
/// button goes down and up at once, so apps that select on the press and apps that select on the
/// release start from the same input time.
pub fn click(run: &CellRun, page: Page, row: i32) -> PresetParams {
    click_at(
        run,
        (OUTPUT.0 / 2, page.row_center(row)),
        page.rows_roi(row, 1),
    )
}

/// A click probe of any toggle: the click at `at` changes the pixels of `roi`, and the next click
/// toggles it back.
pub fn click_at(run: &CellRun, at: (i32, i32), roi: serde_json::Value) -> PresetParams {
    common(run.effort.iterations)
        .set("position", point(at))
        .set("toggle", true)
        .set("hold", "0ms")
        .set("roi", roi)
        .set("detector", "pixels")
        .set("repeats", run.effort.op_repeats)
        .set("timeout", PROBE_TIMEOUT)
}

/// A key probe: `key` changes the pixels of `roi`, and `undo` brings the state back.
pub fn key(run: &CellRun, key: &str, undo: &str, roi: serde_json::Value) -> PresetParams {
    common(run.effort.iterations)
        .set("key", key)
        .set("undo", undo)
        .set("roi", roi)
        .set("detector", "pixels")
        .set("repeats", run.effort.op_repeats)
        .set("timeout", PROBE_TIMEOUT)
}

/// Typing into the focused input at 120 words per minute; the input is cleared between
/// iterations.
pub fn typing(run: &CellRun) -> PresetParams {
    common(run.effort.iterations)
        .set("wpm", 120.0)
        .set("clear", json!(["ctrl+a", "BackSpace"]))
}

/// A resize storm from 640x480 to 1200x800 and back to the maximized window.
pub fn resize_storm(run: &CellRun) -> PresetParams {
    common(run.effort.iterations)
}

/// A scale switch: the iterations alternate between scale 2 and scale 1.
pub fn scale_switch(run: &CellRun) -> PresetParams {
    PresetParams::new()
        .set("iterations", 2 * run.effort.iterations)
        .set("warmup", 0)
        .set("scales", json!([2.0, 1.0]))
}

/// The memory of the app before and after a run of repeated key presses.
#[derive(Debug, Serialize)]
struct Churn {
    key: String,
    cycles: u32,
    before: serde_json::Value,
    after: serde_json::Value,
}

/// Presses `key` `cycles` times, each time after the window settles, and writes the resource
/// samples of the app before and after as `churn.json`. An even count leaves the same state, so
/// the memory difference is what the cycles kept.
pub fn churn(run: &CellRun, key: &str, cycles: u32) -> anyhow::Result<()> {
    let press = |n: u32| -> anyhow::Result<()> {
        let scenario = Scenario::builder("churn")
            .repeat(n, |b| b.key_tap(key).settle(ms(100)))
            .settle(ms(300))
            .sleep(ms(500))
            .build()?;
        let result = run.session.blocking_run(scenario)?.blocking_wait()?;
        anyhow::ensure!(result.ok, "the churn run failed: {:?}", result.error);
        Ok(())
    };
    press(4)?;
    let before = sample(run)?;
    press(cycles)?;
    let after = sample(run)?;
    run.save(
        "churn.json",
        &Churn {
            key: key.to_owned(),
            cycles,
            before,
            after,
        },
    )
}

/// Returns the latest resource sample of the app's process tree.
fn sample(run: &CellRun) -> anyhow::Result<serde_json::Value> {
    let pid = run.app.pid();
    let snapshot = run.session.blocking_resources()?;
    let tree = snapshot
        .processes
        .iter()
        .find(|p| p.pid == pid)
        .context("no resource sample of the app")?;
    Ok(json!({ "process": tree, "gpu": snapshot.gpu }))
}
