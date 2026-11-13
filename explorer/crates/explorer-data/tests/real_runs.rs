//! Loads the runs of the repository's results directory when it exists, and checks that every
//! cell, metric and detail loader reads them.

use std::path::PathBuf;

use explorer_core::api::{MetricRef, RunStatus};
use explorer_data::Store;

fn results() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../results");
    dir.is_dir().then_some(dir)
}

#[test]
fn loads_every_readable_run() {
    let Some(dir) = results() else {
        return;
    };
    let store = Store::new(dir);
    let runs = store.runs();
    assert!(!runs.is_empty());
    for summary in runs.iter().filter(|r| r.status == RunStatus::Ok) {
        let started = std::time::Instant::now();
        let run = store.run(&summary.id).unwrap();
        eprintln!(
            "{}: {} cells, {} metrics, {:?}",
            summary.id,
            run.overview.cells.len(),
            run.catalog.len(),
            started.elapsed()
        );
        let refs: Vec<MetricRef> = run.catalog.iter().map(|m| m.metric.clone()).collect();
        let columns = run.columns(&refs);
        assert!(columns.iter().all(|c| c.cells.len() == c.stats.len()));
        for cell in &run.overview.cells {
            let detail = run.cell_detail(cell.id).unwrap();
            for m in &detail.measurements {
                if m.sections.iter().any(|s| s == "latency") {
                    run.latency_detail(cell.id, &m.name).unwrap();
                }
                if m.sections.iter().any(|s| s == "frame_pacing") {
                    run.frames_detail(cell.id, &m.name).unwrap();
                }
                if m.sections.iter().any(|s| s == "resize") {
                    run.resize_detail(cell.id, &m.name).unwrap();
                }
            }
        }
    }
    let unsupported = runs.iter().filter(|r| r.status != RunStatus::Ok).count();
    eprintln!("{} runs, {unsupported} unsupported", runs.len());
}

#[test]
fn frame_profiles_place_gaps_inside_their_iterations() {
    let Some(dir) = results() else {
        return;
    };
    let store = Store::new(dir);
    let Some(run) = store
        .runs()
        .iter()
        .filter(|r| r.status == RunStatus::Ok)
        .find_map(|r| store.run(&r.id).ok())
    else {
        return;
    };
    let mut points: Vec<(explorer_data::Point, String)> = Vec::new();
    for cell in &run.overview.cells {
        for m in ["scroll", "scroll-fling", "live", "resize"] {
            if !cell.done.iter().any(|d| d == m) {
                continue;
            }
            let point = explorer_data::Point {
                scenario: cell.key.scenario.clone(),
                hz: cell.key.hz,
                rows: cell.key.rows,
            };
            if !points.iter().any(|(p, n)| *p == point && n == m) {
                points.push((point, m.to_owned()));
            }
        }
    }
    let mut lanes = 0;
    for (point, m) in &points {
        for p in run.frame_profiles(point, m) {
            for lane in &p.lanes {
                lanes += 1;
                for g in &lane.gaps {
                    assert!(
                        g.t_ms >= 0.0 && g.t_ms <= lane.window_ms + 1.0,
                        "{point:?} {m}"
                    );
                }
                if lane.complete {
                    let late: u32 = lane.gaps.iter().filter(|g| g.missed > 0).count() as u32;
                    assert!(late <= lane.frames, "{point:?} {m}");
                }
            }
        }
    }
    eprintln!("{} points, {lanes} lanes", points.len());
}
