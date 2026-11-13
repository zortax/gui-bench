//! The health of a run: cells that measured everything, cells that did not run or stopped over
//! the response budget, failed or skipped measurements, and the reports that fail validity
//! checks per framework.

use std::collections::BTreeMap;
use std::sync::Arc;

use explorer_core::api::{CheckStatus, RunOverview};
use leptos::prelude::*;

use crate::palette;
use crate::state::use_explorer;
use crate::widgets::StatusDot;

/// The health card of a run.
#[component]
pub fn Health(overview: Arc<RunOverview>) -> impl IntoView {
    let explorer = use_explorer();
    let cells = &overview.cells;
    let failed: Vec<_> = cells
        .iter()
        .filter(|c| !c.errors.is_empty())
        .cloned()
        .collect();
    let skipped = cells.iter().map(|c| c.skipped.len()).sum::<usize>();
    let stopped: Vec<_> = cells
        .iter()
        .filter(|c| !c.ran() || c.over_budget)
        .cloned()
        .collect();
    let not_run = stopped.iter().filter(|c| !c.ran()).count();
    let missed = cells.iter().filter(|c| c.missed_refreshes).count();
    let mut by_fw: BTreeMap<String, (u32, u32, u32)> = BTreeMap::new();
    for c in cells {
        let slot = by_fw.entry(c.key.framework.clone()).or_default();
        for h in &c.health {
            slot.0 += 1;
            match h.worst {
                CheckStatus::Fail => slot.2 += 1,
                CheckStatus::Warn => slot.1 += 1,
                CheckStatus::Pass => {}
            }
        }
    }
    let mut fws: Vec<String> = by_fw.keys().cloned().collect();
    palette::sort(&mut fws);
    view! {
        <section class="gx-card gx-health">
            <h3 class="gx-chart__title">"Health"</h3>
            <p class="gx-chart__note">
                {format!(
                    "{} cells · {not_run} not run · {} over budget · {missed} missed over 1 % of the refreshes · {} with a failed measurement · {skipped} measurements skipped",
                    cells.len(),
                    stopped.len() - not_run,
                    failed.len()
                )}
            </p>
            <table class="gx-table">
                <thead><tr><th>"framework"</th><th>"reports"</th><th>"pass"</th><th>"warn"</th><th>"fail"</th><th class="gx-table__bar"></th></tr></thead>
                <tbody>
                    {fws.into_iter().map(|fw| {
                        let (n, warn, fail) = by_fw[&fw];
                        let pass = n - warn - fail;
                        let pct = |v: u32| format!("{:.1}%", f64::from(v) / f64::from(n.max(1)) * 100.0);
                        view! {
                            <tr>
                                <td style:color=palette::ink(&fw)><b>{fw.clone()}</b></td>
                                <td>{n}</td>
                                <td>{pass}</td>
                                <td>{warn}</td>
                                <td>{fail}</td>
                                <td class="gx-table__bar">
                                    <span class="gx-stack">
                                        <span data-status="pass" style:width=pct(pass)></span>
                                        <span data-status="warn" style:width=pct(warn)></span>
                                        <span data-status="fail" style:width=pct(fail)></span>
                                    </span>
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
            {(!stopped.is_empty()).then(|| view! {
                <h4 class="gx-health__sub">"Over the response budget"</h4>
                <ul class="gx-health__fails">
                    {stopped.into_iter().map(|c| {
                        let id = c.id;
                        let why = match &c.stopped {
                            Some(why) => format!("not run: {why}"),
                            None => "a response took over a second or never came, so the later steps did not run".to_owned(),
                        };
                        view! {
                            <li>
                                <button type="button" class="gx-btn gx-btn--link" on:click=move |_| explorer.query.set("cell", Some(id.to_string()))>
                                    {c.dir.clone()}
                                </button>
                                <div class="gx-health__err"><StatusDot status=CheckStatus::Warn/>" "{why}</div>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            })}
            {(!failed.is_empty()).then(|| view! {
                <h4 class="gx-health__sub">"Failed measurements"</h4>
                <ul class="gx-health__fails">
                    {failed.into_iter().map(|c| {
                        let id = c.id;
                        view! {
                            <li>
                                <button type="button" class="gx-btn gx-btn--link" on:click=move |_| explorer.query.set("cell", Some(id.to_string()))>
                                    {c.dir.clone()}
                                </button>
                                {c.errors.iter().map(|(m, e)| view! {
                                    <div class="gx-health__err"><StatusDot status=CheckStatus::Fail/>" "<b>{m.clone()}</b>": "{e.clone()}</div>
                                }).collect_view()}
                            </li>
                        }
                    }).collect_view()}
                </ul>
            })}
        </section>
    }
}
