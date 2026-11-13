//! What each scenario measures, in order. A full cell takes the responses first, then the pacing
//! measurements; a pacing cell takes only those.

use fakeland_bench::presets::PresetParams;

use crate::cell::CellRun;
use crate::dashboard::{self, NAV_MESSAGES, NAV_OVERVIEW};
use crate::steps;

use super::{Kind, Part};

/// The key that the search probe types: no customer name holds a `z`, so it hides every order.
const SEARCH_KEY: &str = "z";

/// Runs the measurements of a cell.
pub fn run(kind: Kind, run: &mut CellRun) {
    let full = run.cell.part == Part::Full;
    match kind {
        Kind::App => {
            if full {
                app_responses(run);
            }
            app_pacing(run);
            if full {
                app_messages(run);
            }
        }
        Kind::List => {
            if full {
                list_responses(run);
            }
            list_pacing(run);
        }
    }
}

/// Runs a hover probe, and the same probe with app marks for an app that sends them.
fn hover(run: &mut CellRun, name: &str, target: dashboard::Target) {
    run.step(name, |r| {
        let params = steps::hover(r, &target, "pixels");
        steps::measure(r, name, "hover-latency", &params).map(drop)
    });
    if run.fw.marks {
        let marks = format!("{name}-marks");
        run.step(&marks, |r| {
            let params = steps::hover(r, &target, "app_mark");
            steps::measure(r, &marks, "hover-latency", &params).map(drop)
        });
    }
}

/// Runs a click probe.
fn click(run: &mut CellRun, name: &str, toggle: dashboard::Toggle) {
    run.step(name, |r| {
        steps::measure(r, name, "click-latency", &steps::click(r, &toggle)).map(drop)
    });
}

/// The overview at rest and its responses: hover a nav item and an order row, click the compare
/// switch, a status filter, the sort title and the export menu, and type into the search.
fn app_responses(run: &mut CellRun) {
    run.step("idle", |r| {
        steps::measure(r, "idle", "idle", &steps::idle()).map(drop)
    });
    hover(run, "hover-nav", dashboard::hover_nav());
    hover(run, "hover-row", dashboard::hover_row());
    click(run, "click-compare", dashboard::click_compare());
    click(run, "click-filter", dashboard::click_filter());
    click(run, "click-sort", dashboard::click_sort());
    click(run, "click-menu", dashboard::click_menu());
    run.step("menu-shot", |r| {
        let at = dashboard::click_menu().at;
        steps::click_once(r, at, true)?;
        let shot = steps::screenshot(r, "screenshot-menu").map(drop);
        steps::click_once(r, at, true)?;
        steps::park(r, dashboard::PARK)?;
        shot
    });
    run.step("key-search", |r| {
        steps::click_once(r, dashboard::SEARCH, true)?;
        let params = steps::key(r, SEARCH_KEY, "BackSpace", dashboard::filtered_roi());
        let done = steps::measure(r, "key-search", "key-latency", &params).map(drop);
        steps::park(r, dashboard::PARK)?;
        done
    });
}

/// The pacing of the overview: three kinds of scroll over the orders, a resize storm, and the
/// live updates with the spinner.
fn app_pacing(run: &mut CellRun) {
    let (pointer, roi) = (dashboard::SCROLL_POINTER, dashboard::scroll_roi());
    scrolls(run, pointer, roi);
    run.step("resize", |r| {
        steps::measure(r, "resize", "resize-storm", &steps::resize_storm(r)).map(drop)
    });
    run.step("live", |r| {
        steps::click_once(r, dashboard::LIVE, false)?;
        let _ = steps::screenshot(r, "screenshot-live");
        let (params, seconds) = steps::pacing(r);
        let done = steps::measure_pacing(r, "live", "pacing", &params, seconds);
        steps::click_once(r, dashboard::LIVE, true)?;
        steps::park(r, dashboard::PARK)?;
        done
    });
}

/// Builds the parameters of a scroll for a pointer and a motion region, with its length in
/// seconds.
type ScrollParams = fn(&CellRun, (i32, i32), serde_json::Value) -> (PresetParams, f64);

/// The glide, the wheel scroll and the fling, each when the rows are tall enough for it.
fn scrolls(run: &mut CellRun, pointer: (i32, i32), roi: serde_json::Value) {
    let rows = run.cell.rows;
    // A wheel moves the content in notches with rests between them, so its refreshes without
    // a new frame say nothing about the rate the app holds.
    let kinds: [(&str, &str, f64, bool, ScrollParams); 3] = [
        (
            "scroll",
            "scroll-glide",
            steps::GLIDE_VELOCITY * steps::GLIDE_SECONDS,
            true,
            steps::glide,
        ),
        (
            "scroll-wheel",
            "scroll-glide",
            steps::WHEEL_VELOCITY * steps::GLIDE_SECONDS,
            false,
            steps::wheel,
        ),
        (
            "scroll-fling",
            "fling",
            steps::FLING_TRAVEL,
            true,
            steps::fling,
        ),
    ];
    for (name, preset, travel, paced, params) in kinds {
        if !dashboard::scrolls(rows, travel) {
            run.skip(name, "the rows are shorter than the scroll");
            continue;
        }
        let roi = roi.clone();
        run.step(name, |r| {
            let (params, seconds) = params(r, pointer, roi);
            if paced {
                steps::measure_pacing(r, name, preset, &params, seconds)
            } else {
                steps::measure(r, name, preset, &params).map(drop)
            }
        });
    }
    // The later steps probe and show the top of the content.
    if let Err(e) = steps::scroll_to_top(run, pointer) {
        eprintln!("    the content did not scroll back to the top: {e:#}");
    }
}

/// The page switches and the messages page: the click that mounts the messages, the click that
/// mounts the overview, a key in the search input beside the messages, and a resize storm over
/// their wrapping text.
fn app_messages(run: &mut CellRun) {
    click(
        run,
        "page-messages",
        dashboard::click_page(NAV_OVERVIEW, NAV_MESSAGES),
    );
    run.step("to-messages", |r| {
        steps::click_once(r, dashboard::nav(NAV_MESSAGES), true)?;
        steps::park(r, dashboard::PARK)?;
        steps::screenshot(r, "screenshot-messages").map(drop)
    });
    click(
        run,
        "page-overview",
        dashboard::click_page(NAV_MESSAGES, NAV_OVERVIEW),
    );
    run.step("key-type", |r| {
        steps::click_once(r, dashboard::SEARCH, true)?;
        let params = steps::key(r, "a", "BackSpace", dashboard::search_roi());
        let done = steps::measure(r, "key-type", "key-latency", &params).map(drop);
        steps::park(r, dashboard::PARK)?;
        done
    });
    run.step("resize-messages", |r| {
        steps::measure(
            r,
            "resize-messages",
            "resize-storm",
            &steps::resize_storm(r),
        )
        .map(drop)
    });
}

/// The list at rest and its responses: hover a row and sort the data.
fn list_responses(run: &mut CellRun) {
    run.step("idle", |r| {
        steps::measure(r, "idle", "idle", &steps::idle()).map(drop)
    });
    hover(run, "hover-row", dashboard::hover_list_row());
    click(run, "click-sort", dashboard::click_list_sort());
}

/// The pacing of the list: the three kinds of scroll.
fn list_pacing(run: &mut CellRun) {
    scrolls(run, dashboard::SCROLL_POINTER, dashboard::list_scroll_roi());
}
