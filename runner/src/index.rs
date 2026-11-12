//! The index of a run, `run.json`: the settings, the revisions of the checkouts, and the outcome
//! of every cell. A resumed run keeps the cells of the earlier index that the new matrix does not
//! name, so a run can grow by scenarios; `history` lists the settings of every invocation.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use crate::scenarios::{self, Effort};
use crate::{cell, frameworks, layout, steps};

/// The format of `run.json` and the run directory.
const FORMAT: u32 = 2;

/// The index of a run while it runs.
pub struct Index {
    path: PathBuf,
    started_unix: u64,
    scenarios: Vec<String>,
    frameworks: Vec<String>,
    effort: Effort,
    a11y: bool,
    history: Vec<Value>,
    /// The cells of the earlier index outside the new matrix.
    kept: Vec<Value>,
}

/// The settings of one invocation of the runner.
pub struct Invocation<'a> {
    pub scenarios: &'a [String],
    pub frameworks: &'a [String],
    pub effort: Effort,
    pub started_unix: u64,
    /// The sessions run an accessibility bus.
    pub a11y: bool,
}

impl Index {
    /// Starts the index of `run_dir`. With `resume`, it reads the earlier index and keeps its
    /// cells whose directories are not in `matrix`.
    pub fn new(
        run_dir: &Path,
        run: &Invocation<'_>,
        resume: bool,
        matrix: &HashSet<String>,
    ) -> Self {
        let path = run_dir.join("run.json");
        let invocation = json!({
            "started_unix": run.started_unix,
            "scenarios": run.scenarios,
            "frameworks": run.frameworks,
            "effort": run.effort,
            "accessibility": run.a11y,
            "revisions": revisions(),
        });
        let mut index = Self {
            path,
            started_unix: run.started_unix,
            scenarios: run.scenarios.to_vec(),
            frameworks: run.frameworks.to_vec(),
            effort: run.effort,
            a11y: run.a11y,
            history: Vec::new(),
            kept: Vec::new(),
        };
        let earlier = resume
            .then(|| std::fs::read_to_string(&index.path).ok())
            .flatten()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok());
        if let Some(earlier) = earlier {
            index.merge(&earlier, matrix);
        }
        index.history.push(invocation);
        index
    }

    /// Takes the start, the names, the history and the cells outside `matrix` of an earlier index.
    fn merge(&mut self, earlier: &Value, matrix: &HashSet<String>) {
        if let Some(t) = earlier["started_unix"].as_u64() {
            self.started_unix = t;
        }
        self.scenarios = union(names(&earlier["scenarios"]), &self.scenarios);
        self.frameworks = union(names(&earlier["frameworks"]), &self.frameworks);
        self.history = match earlier["history"].as_array() {
            Some(history) => history.clone(),
            // An index from before `history` describes its one invocation at the top.
            None => vec![json!({
                "started_unix": earlier["started_unix"],
                "scenarios": earlier["scenarios"],
                "frameworks": earlier["frameworks"],
                "effort": earlier["effort"],
                "revisions": earlier["revisions"],
            })],
        };
        let cells = earlier["cells"].as_array().into_iter().flatten();
        self.kept = cells
            .filter(|c| c["dir"].as_str().is_some_and(|d| !matrix.contains(d)))
            .cloned()
            .collect();
    }

    /// Writes `run.json` with the kept cells and the cells of this invocation.
    pub fn write(&self, cells: &[Value]) -> anyhow::Result<()> {
        let all: Vec<&Value> = self.kept.iter().chain(cells).collect();
        let index = json!({
            "format": FORMAT,
            "started_unix": self.started_unix,
            "scenarios": self.scenarios,
            "frameworks": self.frameworks,
            "effort": self.effort,
            "accessibility": self.a11y,
            "output": { "width": layout::OUTPUT.0, "height": layout::OUTPUT.1 },
            "row_height": layout::ROW_HEIGHT,
            "header_height": layout::HEADER_HEIGHT,
            "scroll": { "velocity": steps::SCROLL_VELOCITY, "seconds": steps::SCROLL_SECONDS },
            "tick_ms": scenarios::TICK_MS,
            "revisions": revisions(),
            "history": self.history,
            "cells": all,
        });
        cell::save(&self.path, &index)
    }
}

/// Returns the strings of a JSON array.
fn names(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

/// Returns `first` followed by the names of `then` that it lacks.
fn union(mut first: Vec<String>, then: &[String]) -> Vec<String> {
    for name in then {
        if !first.contains(name) {
            first.push(name.clone());
        }
    }
    first
}

/// Returns the revisions of this checkout and of the fakeland and zgui checkouts beside it.
fn revisions() -> Value {
    let root = frameworks::root();
    json!({
        "gui-bench": revision(&root),
        "fakeland": revision(&root.join("../fakeland")),
        "zgui": revision(&root.join("../zgui-fakeland-marks")),
    })
}

/// Returns the short commit of a checkout, with `+dirty` when it has changes.
fn revision(path: &Path) -> Option<String> {
    let git = |args: &[&str]| {
        // `--no-optional-locks`: a status refresh takes no `index.lock`, so a commit that runs
        // while the matrix runs never meets a held lock.
        Command::new("git")
            .arg("--no-optional-locks")
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
