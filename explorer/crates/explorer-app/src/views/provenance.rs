//! The provenance of a run: revisions, host, output, effort, framework versions, the invocations
//! that wrote it and the validity of its reports.

use std::sync::Arc;

use explorer_core::api::{CheckStatus, RunOverview};
use explorer_core::units::duration;
use leptos::prelude::*;

use crate::widgets::{Badge, LocalTime, StatusDot};

/// The provenance panel of a run.
#[component]
pub fn Provenance(overview: Arc<RunOverview>) -> impl IntoView {
    let p = overview.provenance.clone();
    let s = overview.summary.clone();
    let env = p.environment.clone().unwrap_or_default();
    let span = p
        .started_unix
        .zip(p.modified_unix)
        .map(|(a, b)| duration(b.saturating_sub(a)));
    let e = &p.effort;
    let effort = [
        e.iterations.map(|v| format!("{v} iterations")),
        e.hover_repeats.map(|v| format!("{v} hover repeats")),
        e.op_repeats.map(|v| format!("{v} op repeats")),
        e.churn_cycles.map(|v| format!("{v} churn cycles")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ");
    view! {
        <div class="gx-prov">
            <dl class="gx-kv">
                <dt>"run"</dt>
                <dd>
                    <b>{s.id.clone()}</b>
                    {s.aliases.iter().map(|a| view! { " " <Badge text=a.clone() tone="info"/> }).collect_view()}
                </dd>
                <dt>"started"</dt>
                <dd>
                    {p.started_unix.map(|t| view! { <LocalTime unix=t/> })}
                    {span.map(|d| format!(" · ran {d}"))}
                </dd>
                <dt>"revisions"</dt>
                <dd class="gx-prov__revs">
                    {p.revisions.iter().map(|r| view! {
                        <span class="gx-prov__rev">
                            {r.name.clone()} " "
                            <code>{r.commit.clone().unwrap_or_else(|| "unknown".to_owned())}</code>
                            {r.dirty.then(|| view! { " " <Badge text="dirty" tone="warn" title="The checkout had uncommitted changes"/> })}
                        </span>
                    }).collect_view()}
                </dd>
                <dt>"host"</dt>
                <dd>{format!("{} · {}", env.cpu, env.gpu)}</dd>
                <dt>"system"</dt>
                <dd>{format!("{} · kernel {} · governor {} · {} · fakeland {}", env.driver, env.kernel, env.governor, env.renderer, env.fakeland_version)}</dd>
                <dt>"output"</dt>
                <dd>
                    {p.output.map(|(w, h)| format!("{w}×{h} at scale 1"))}
                    {p.monitor.clone().map(|m| format!(" · {m}"))}
                    {p.row_height.map(|r| format!(" · rows {r} px"))}
                </dd>
                <dt>"effort"</dt>
                <dd>{effort}</dd>
                {p.scroll.map(|(v, s)| view! { <dt>"scroll"</dt><dd>{format!("{v} px/s for {s} s")}</dd> })}
                {p.tick_ms.map(|t| view! { <dt>"ticker"</dt><dd>{format!("every {t} ms")}</dd> })}
                {p.accessibility.map(|on| view! {
                    <dt>"accessibility"</dt>
                    <dd>{if on { "the sessions run an accessibility bus" } else { "off: no accessibility bus" }}</dd>
                })}
                {(p.history.len() > 1).then(|| view! {
                    <dt>"history"</dt>
                    <dd class="gx-prov__history">
                        {p.history.iter().map(|h| view! {
                            <span>
                                {h.started_unix.map(|t| view! { <LocalTime unix=t/> })}
                                " · " {h.scenarios.join(", ")}
                                {h.revisions.iter().filter(|r| r.dirty).map(|r| view! { " " <Badge text=format!("{} dirty", r.name) tone="warn" title="The checkout had uncommitted changes"/> }).collect_view()}
                                {h.accessibility.filter(|a| *a).map(|_| view! { " " <Badge text="a11y" tone="info" title="The sessions ran an accessibility bus"/> })}
                            </span>
                        }).collect_view()}
                    </dd>
                })}
                <dt>"versions"</dt>
                <dd>
                    {if p.versions.is_empty() {
                        view! { <span class="gx-muted">"not recorded in this run"</span> }.into_any()
                    } else {
                        view! {
                            <span class="gx-prov__versions">
                                {p.versions.iter().map(|v| view! {
                                    <span title=v.source.clone().unwrap_or_default()>
                                        {v.framework.clone()} " " <code>{format!("{} {}", v.krate, v.version)}</code>
                                    </span>
                                }).collect_view()}
                            </span>
                        }.into_any()
                    }}
                </dd>
                <dt>"validity"</dt>
                <dd>
                    {format!("{} of {} reports valid", p.validity.valid, p.validity.reports)}
                    <span class="gx-prov__checks">
                        {p.validity.checks.iter().map(|c| view! {
                            <span>
                                <StatusDot status={if c.fail > 0 { CheckStatus::Fail } else { CheckStatus::Warn }}/>
                                {format!(" {} ", c.check)}
                                <span class="gx-muted">{format!("{} fail · {} warn", c.fail, c.warn)}</span>
                            </span>
                        }).collect_view()}
                    </span>
                </dd>
            </dl>
        </div>
    }
}
