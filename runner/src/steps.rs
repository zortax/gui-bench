//! The measurements that scenarios share. Each one runs a fakeland bench preset with parameters
//! from the cell, adds the resource and frame-cost analyzers, and writes the report as
//! `<name>.json`. Apps with app marks also get `phases-<name>.json`. A measurement that the host
//! disturbs runs again.

use anyhow::Context;
use fakeland::scenario::Position;
use fakeland::scenario::prelude::*;
use fakeland::{CaptureOptions, CaptureTarget};
use fakeland_bench::analyzers::{FrameCost, Resources};
use fakeland_bench::presets::{self, PresetParams};
use fakeland_bench::{Bench, BenchRun};
use serde::Serialize;
use serde_json::{Value, json};

use crate::cell::CellRun;
use crate::dashboard::{Target, Toggle};
use crate::phases;

/// The velocity of the scroll glide in logical pixels per second, and its length in seconds.
pub const GLIDE_VELOCITY: f64 = 800.0;
pub const GLIDE_SECONDS: f64 = 1.0;
/// The start velocity of the fling in logical pixels per second. With the decay of the preset it
/// travels about 2600 px.
pub const FLING_VELOCITY: f64 = 8000.0;
pub const FLING_TRAVEL: f64 = 2600.0;
/// The velocity of the wheel scroll in logical pixels per second: about 13 detents a second.
pub const WHEEL_VELOCITY: f64 = 200.0;
/// The start velocity of the fling that brings the content back to its top, in logical pixels
/// per second: far more than any measured scroll moves it.
const TOP_VELOCITY: f64 = 60_000.0;
/// The scroll events of a finger per refresh, so every frame can consume the same scroll.
const EVENTS_PER_REFRESH: f64 = 4.0;
/// How long the idle measurement lasts.
const IDLE_SECONDS: f64 = 3.0;
/// How long one iteration of the live measurement lasts.
const LIVE_SECONDS: f64 = 2.0;
/// The probe timeout: the rebuild of a large page can take a while.
const PROBE_TIMEOUT: &str = "5s";
/// The median response in milliseconds above which a cell stops: its other measurements would
/// take minutes and say no more than that the app is past use at this size.
const STOP_MS: f64 = 1000.0;
/// The share of missed refreshes above which a pacing measurement counts as missing its rate.
const MISSED_SHARE: f64 = 0.01;

/// Builds a preset bench, adds the resource and frame-cost analyzers, and runs it. A run whose
/// report is invalid, because the compositor woke late or the host was busy, runs again, at most
/// `effort.retries` times; the last run counts.
pub fn bench(run: &CellRun, preset: &str, params: &PresetParams) -> anyhow::Result<BenchRun> {
    let mut attempt = 0;
    loop {
        let bench = presets::by_name(preset)
            .with_context(|| format!("no preset {preset}"))?
            .bench(params)?;
        let result = with_cost(bench).blocking_run(&run.session)?;
        if result.report_ref().validity.valid || attempt >= run.effort.retries {
            return Ok(result);
        }
        attempt += 1;
        eprintln!(
            "      the host disturbed the measurement; attempt {}",
            attempt + 1
        );
    }
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

/// Runs a preset bench, writes its report as `<name>.json` and returns the report as JSON.
pub fn measure(
    run: &CellRun,
    name: &str,
    preset: &str,
    params: &PresetParams,
) -> anyhow::Result<Value> {
    let result = bench(run, preset, params)?;
    let report = result.report();
    run.save(&format!("{name}.json"), &report)?;
    if run.fw.marks {
        run.save(&format!("phases-{name}.json"), &phases_of(&result))?;
    }
    let report = serde_json::to_value(&report)?;
    let slow = metric(&report, "latency", "input_to_commit_p50").is_some_and(|ms| ms > STOP_MS);
    let unanswered = metric(&report, "latency", "timeouts").is_some_and(|n| n > 0.0);
    if slow || unanswered {
        run.over.set(true);
    }
    Ok(report)
}

/// Runs a pacing measurement and notes in the cell when it misses more than `MISSED_SHARE` of the
/// refreshes of its `seconds`.
pub fn measure_pacing(
    run: &CellRun,
    name: &str,
    preset: &str,
    params: &PresetParams,
    seconds: f64,
) -> anyhow::Result<()> {
    let report = measure(run, name, preset, params)?;
    if let Some(missed) = metric(&report, "frame_pacing", "missed_vblanks")
        && missed > MISSED_SHARE * seconds * run.cell.hz
    {
        run.missed.set(true);
    }
    Ok(())
}

/// Returns the median over the iterations of a metric of a report section.
pub fn metric(report: &Value, section: &str, name: &str) -> Option<f64> {
    report["sections"]
        .as_array()?
        .iter()
        .find(|s| s["name"] == section)?["metrics"]
        .as_array()?
        .iter()
        .find(|m| m["name"] == name)?["median"]
        .as_f64()
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
        .set("duration", format!("{IDLE_SECONDS}s"))
        .set("suspend", true)
        .set("capture", true)
        .set("downscale", 2)
}

/// The pacing measurement of the live updates, and its length in seconds. It has no warmup: the
/// app has run live for a second when it starts.
pub fn pacing(run: &CellRun) -> (PresetParams, f64) {
    let params = PresetParams::new()
        .set("iterations", run.effort.iterations)
        .set("warmup", 0)
        .set("duration", format!("{LIVE_SECONDS}s"));
    (params, LIVE_SECONDS)
}

/// A hover probe: the pointer enters `target.at` from `target.away`, and the pixels of
/// `target.roi` tell the change. `detector` is `pixels` or `app_mark`.
pub fn hover(run: &CellRun, target: &Target, detector: &str) -> PresetParams {
    common(run.effort.iterations)
        .set("position", point(target.at))
        .set("away", point(target.away))
        .set("roi", target.roi.clone())
        .set("detector", detector)
        .set("repeats", run.effort.hover_repeats)
        .set("timeout", "2s")
}

/// Returns a window point as preset JSON.
fn point((x, y): (i32, i32)) -> Value {
    json!({ "x": x, "y": y })
}

/// A click probe: the click at `toggle.at` changes the pixels of `toggle.roi`, and a click
/// between repeats brings the state back. The button goes down and up at once, so apps that act
/// on the press and apps that act on the release start from the same input time.
pub fn click(run: &CellRun, toggle: &Toggle) -> PresetParams {
    let params = common(run.effort.iterations)
        .set("position", point(toggle.at))
        .set("toggle", true)
        .set("hold", "0ms")
        .set("roi", toggle.roi.clone())
        .set("detector", "pixels")
        .set("repeats", run.effort.op_repeats)
        .set("timeout", PROBE_TIMEOUT);
    match toggle.undo {
        Some(undo) => params.set("undo_position", point(undo)),
        None => params,
    }
}

/// A key probe: `key` changes the pixels of `roi`, and `undo` brings the state back.
pub fn key(run: &CellRun, key: &str, undo: &str, roi: Value) -> PresetParams {
    common(run.effort.iterations)
        .set("key", key)
        .set("undo", undo)
        .set("roi", roi)
        .set("detector", "pixels")
        .set("repeats", run.effort.op_repeats)
        .set("timeout", PROBE_TIMEOUT)
}

/// The parameters of a scroll with the pointer at `pointer` and motion estimation in `roi`.
fn scroll(run: &CellRun, pointer: (i32, i32), roi: Value) -> PresetParams {
    PresetParams::new()
        .set("position", point(pointer))
        .set("roi", roi)
        .set("iterations", run.effort.iterations)
        .set("warmup", 1)
        .set("rate_hz", run.cell.hz * EVENTS_PER_REFRESH)
}

/// The scroll glide: a finger at 800 px/s for a second with no kinetic tail, four events per
/// refresh. Returns the
/// parameters and the length in seconds.
pub fn glide(run: &CellRun, pointer: (i32, i32), roi: Value) -> (PresetParams, f64) {
    let params = scroll(run, pointer, roi)
        .set("source", json!({ "type": "finger", "kinetic": false }))
        .set("velocity", GLIDE_VELOCITY)
        .set("duration", format!("{GLIDE_SECONDS}s"));
    (params, GLIDE_SECONDS)
}

/// The wheel scroll: a mouse wheel that turns about 13 detents in a second, so the content
/// moves the way the framework moves it for a wheel, with its smoothing if it has one.
pub fn wheel(run: &CellRun, pointer: (i32, i32), roi: Value) -> (PresetParams, f64) {
    let params = scroll(run, pointer, roi)
        .set("source", json!({ "type": "wheel", "hires": false }))
        .set("velocity", WHEEL_VELOCITY)
        .set("duration", format!("{GLIDE_SECONDS}s"));
    (params, GLIDE_SECONDS)
}

/// The fling: a finger whose velocity decays from 8000 px/s over a second, which brings many
/// new rows into view per frame.
pub fn fling(run: &CellRun, pointer: (i32, i32), roi: Value) -> (PresetParams, f64) {
    let params = scroll(run, pointer, roi)
        .set("velocity", FLING_VELOCITY)
        .set("duration", format!("{GLIDE_SECONDS}s"));
    (params, GLIDE_SECONDS)
}

/// A resize storm from 640x480 to 1200x800 and back to the maximized window.
pub fn resize_storm(run: &CellRun) -> PresetParams {
    common(run.effort.iterations)
}

/// Clicks a point and waits until the window settles, or for `wait` when the app keeps drawing.
pub fn click_once(run: &CellRun, at: (i32, i32), settles: bool) -> anyhow::Result<()> {
    let position = Position::output(f64::from(at.0), f64::from(at.1));
    let builder =
        Scenario::builder("click").step(PointerAction::click(Some(position), Button::Left));
    let builder = if settles {
        builder.settle(ms(300))
    } else {
        builder.sleep(ms(1000))
    };
    let result = run
        .session
        .blocking_run(builder.build()?)?
        .blocking_wait()?;
    anyhow::ensure!(result.ok, "the click failed: {:?}", result.error);
    Ok(())
}

/// Flings the content under `pointer` back to its top and waits until the window settles. The
/// scrolls go down and up by the same input, but a framework that scales scroll input reaches
/// the end of the content on the way down, so it does not come back to the top by itself.
pub fn scroll_to_top(run: &CellRun, pointer: (i32, i32)) -> anyhow::Result<()> {
    let mut fling = Fling::new(-TOP_VELOCITY);
    fling.at = Some(Position::output(f64::from(pointer.0), f64::from(pointer.1)));
    fling.duration = ms(400);
    let scenario = Scenario::builder("to-top")
        .fling(fling)
        .settle(ms(300))
        .build()?;
    let result = run.session.blocking_run(scenario)?.blocking_wait()?;
    anyhow::ensure!(
        result.ok,
        "the scroll to the top failed: {:?}",
        result.error
    );
    Ok(())
}

/// Moves the pointer to a point and waits until the window settles.
pub fn park(run: &CellRun, at: (i32, i32)) -> anyhow::Result<()> {
    run.session
        .blocking_pointer_move(Position::output(f64::from(at.0), f64::from(at.1)))?;
    run.session.blocking_wait_settled(
        fakeland::WindowRef::Focused,
        fakeland_types::Nanos::from_millis(300),
        fakeland_types::Nanos::from_secs(10),
    )?;
    Ok(())
}

/// Writes a screenshot of the window as `<name>.png`.
pub fn screenshot(run: &CellRun, name: &str) -> anyhow::Result<fakeland::Screenshot> {
    let shot = run.session.blocking_screenshot(
        CaptureTarget::default(),
        CaptureOptions {
            include_cursor: false,
            max_dim: None,
        },
    )?;
    std::fs::write(
        run.out.join(format!("{name}.png")),
        shot.image.encode_png()?,
    )?;
    Ok(shot)
}
