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
                if m.sections.iter().any(|s| s == "typing") {
                    run.typing_detail(cell.id, &m.name).unwrap();
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
