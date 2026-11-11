//! Runs the GUI benchmark matrix under fakeland. For every framework, refresh rate and row count,
//! the runner starts a headless session, launches the framework's rows app, and measures startup,
//! hover latency and scroll smoothness. Results go to `results/<run>/` as JSON, with `run.json`
//! as the index; `results/latest` links to the newest run.

mod cell;
mod frameworks;
mod matrix;
mod phases;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail};
use clap::Parser;
use serde_json::json;

use crate::frameworks::Framework;
use crate::matrix::{Cell, Effort};

/// Runs the GUI benchmark matrix under fakeland.
#[derive(Debug, Parser)]
struct Args {
    /// The frameworks, comma separated.
    #[arg(long, value_delimiter = ',', default_value = "zgui,gpui-kit")]
    frameworks: Vec<String>,
    /// The row counts, comma separated.
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "10,100,300,1000,3000,10000"
    )]
    rows: Vec<u32>,
    /// The output refresh rates in Hz, comma separated.
    #[arg(long, value_delimiter = ',', default_value = "60,144")]
    hz: Vec<f64>,
    /// Hover repeats per iteration.
    #[arg(long, default_value_t = 12)]
    repeats: u32,
    /// Measured iterations of every scenario.
    #[arg(long, default_value_t = 3)]
    iterations: u32,
    /// A small matrix for a quick check: 100 and 3000 rows at 60 Hz with fewer repeats.
    #[arg(long)]
    quick: bool,
    /// The run directory. The default is `results/run-<unix time>`.
    #[arg(long)]
    out: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();
    if args.quick {
        args.rows = vec![100, 3000];
        args.hz = vec![60.0];
        args.repeats = 6;
        args.iterations = 2;
    }
    let root = frameworks::root();
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
        iterations: args.iterations,
    };
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let run_dir = args
        .out
        .clone()
        .unwrap_or_else(|| root.join("results").join(format!("run-{stamp}")));
    std::fs::create_dir_all(&run_dir)?;
    link_latest(&root, &run_dir);

    let total = selected.len() * args.hz.len() * args.rows.len();
    let mut cells = Vec::new();
    let mut index = 0;
    for (fw, binary) in &selected {
        for &hz in &args.hz {
            for &rows in &args.rows {
                index += 1;
                let cell = Cell { hz, rows };
                eprintln!("[{index}/{total}] {} {hz} Hz, {rows} rows", fw.name);
                cells.push(cell::run(fw, binary, cell, effort, &run_dir));
                write_index(&run_dir, &args, effort, stamp, &cells)?;
            }
        }
    }
    let failed: usize = cells.iter().map(|c| c.errors.len()).sum();
    eprintln!("results: {} ({failed} failed scenarios)", run_dir.display());
    Ok(())
}

fn write_index(
    run_dir: &Path,
    args: &Args,
    effort: Effort,
    stamp: u64,
    cells: &[cell::CellOutcome],
) -> anyhow::Result<()> {
    let root = frameworks::root();
    let index = json!({
        "started_unix": stamp,
        "frameworks": args.frameworks,
        "rows": args.rows,
        "hz": args.hz,
        "effort": effort,
        "output": { "width": matrix::OUTPUT.0, "height": matrix::OUTPUT.1 },
        "row_height": matrix::ROW_HEIGHT,
        "scroll": { "velocity": matrix::SCROLL_VELOCITY, "seconds": matrix::SCROLL_SECONDS },
        "revisions": {
            "gui-bench": revision(&root),
            "fakeland": revision(&root.join("../fakeland")),
            "zgui": revision(&root.join("../zgui-fakeland-marks")),
        },
        "cells": cells,
    });
    cell::save(&run_dir.join("run.json"), &index)
}

/// Returns the short commit of a checkout, with `+dirty` when it has changes.
fn revision(path: &Path) -> Option<String> {
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
    };
    let commit = git(&["rev-parse", "--short", "HEAD"])?;
    let dirty = git(&["status", "--porcelain"]).is_some_and(|s| !s.is_empty());
    Some(if dirty {
        format!("{commit}+dirty")
    } else {
        commit
    })
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
