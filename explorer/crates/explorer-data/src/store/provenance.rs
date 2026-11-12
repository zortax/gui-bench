//! The provenance of a run: the index header, the host from the first report, the faked output,
//! the framework versions and invocations the index records, and the validity tally of every
//! report.

use std::collections::BTreeMap;
use std::time::UNIX_EPOCH;

use explorer_core::api::{
    CheckStatus, CheckTally, Effort, FrameworkVersion, Invocation, Provenance, Revision,
    ValidityTally,
};

use super::load::LoadedCell;
use crate::catalog::Entry;
use crate::report::RunIndex;

/// Returns the provenance of a loaded run.
pub(super) fn collect(index: &RunIndex, entry: &Entry, cells: &[LoadedCell]) -> Provenance {
    let reports = || cells.iter().flat_map(|c| c.reports.iter().map(|(_, r)| r));
    let environment = reports()
        .find_map(|r| r.environment.as_ref())
        .map(|e| e.to_api());
    let monitor = reports()
        .find_map(|r| r.config.as_ref()?.outputs.first().cloned())
        .map(|o| {
            let make = [o.make, o.model]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ");
            match o.name {
                Some(name) if !make.is_empty() => format!("{name} · {make}"),
                Some(name) => name,
                None => make,
            }
        });
    let effort_of = |k: &str| {
        index
            .effort
            .get(k)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
    };
    let revisions = revisions_of(&index.revisions);
    let history = index
        .history
        .iter()
        .map(|h| Invocation {
            started_unix: h.started_unix,
            scenarios: h.scenarios.clone(),
            frameworks: h.frameworks.clone(),
            revisions: revisions_of(&h.revisions),
            accessibility: h.accessibility,
        })
        .collect();
    let versions = index
        .framework_versions
        .iter()
        .filter_map(|(fw, v)| {
            Some(FrameworkVersion {
                framework: fw.clone(),
                krate: v.krate.clone().unwrap_or_else(|| fw.clone()),
                version: v.version.clone()?,
                source: v.source.clone(),
            })
        })
        .collect();
    Provenance {
        started_unix: index.started_unix,
        modified_unix: entry
            .modified
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs()),
        revisions,
        effort: Effort {
            iterations: effort_of("iterations"),
            hover_repeats: effort_of("hover_repeats"),
            op_repeats: effort_of("op_repeats"),
            churn_cycles: effort_of("churn_cycles"),
        },
        output: index.output.as_ref().map(|o| (o.width, o.height)),
        monitor,
        row_height: index.row_height,
        scroll: index.scroll.as_ref().map(|s| (s.velocity, s.seconds)),
        tick_ms: index.tick_ms,
        scenarios: entry.summary.scenarios.clone(),
        frameworks: entry.summary.frameworks.clone(),
        environment,
        versions,
        validity: tally(cells),
        accessibility: index.accessibility,
        history,
    }
}

fn revisions_of(map: &BTreeMap<String, Option<String>>) -> Vec<Revision> {
    map.iter()
        .map(|(name, rev)| Revision {
            name: name.clone(),
            commit: rev
                .as_deref()
                .map(|r| r.trim_end_matches("+dirty").to_owned()),
            dirty: rev.as_deref().is_some_and(|r| r.ends_with("+dirty")),
        })
        .collect()
}

fn tally(cells: &[LoadedCell]) -> ValidityTally {
    let mut out = ValidityTally::default();
    let mut checks: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for (_, report) in cells.iter().flat_map(|c| &c.reports) {
        let Some(validity) = &report.validity else {
            continue;
        };
        out.reports += 1;
        out.valid += u32::from(validity.valid);
        for check in &validity.checks {
            let slot = checks.entry(check.name.clone()).or_default();
            match check.status() {
                CheckStatus::Warn => slot.0 += 1,
                CheckStatus::Fail => slot.1 += 1,
                CheckStatus::Pass => {}
            }
        }
    }
    out.checks = checks
        .into_iter()
        .filter(|(_, (w, f))| w + f > 0)
        .map(|(check, (warn, fail))| CheckTally { check, warn, fail })
        .collect();
    out.checks
        .sort_by(|a, b| b.fail.cmp(&a.fail).then(b.warn.cmp(&a.warn)));
    out
}
