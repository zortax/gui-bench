//! One measurement of a cell: its validity checks, its iterations and parameters, and the
//! charts of its raw series by report section.

use explorer_core::api::MeasurementDetail;
use explorer_core::units::short;
use leptos::prelude::*;

use super::latency::LatencyCharts;
use super::series::{FrameCharts, PhaseChart, ResizeCharts};
use crate::widgets::StatusDot;

/// The panel of one measurement.
#[component]
pub fn MeasurementPanel(cell: u32, measurement: MeasurementDetail) -> impl IntoView {
    let m = measurement;
    let has = |s: &str| m.sections.iter().any(|x| x == s);
    let latency = has("latency");
    let frames = has("scroll_smoothness") || has("frame_pacing");
    let resize = has("resize");
    let name = m.name.clone();
    view! {
        <div class="gx-drawer__panel">
            <div class="gx-drawer__cols">
                <section>
                    <h4>"Validity" {m.preset.clone().map(|p| view! { <span class="gx-muted">{format!(" · {p}")}</span> })}</h4>
                    <table class="gx-table">
                        <tbody>
                            {m.checks.iter().map(|c| view! {
                                <tr>
                                    <td><StatusDot status=c.status/></td>
                                    <td>{c.name.clone()}</td>
                                    <td class="gx-muted">{c.message.clone()}</td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                </section>
                <section>
                    <h4>"Iterations"</h4>
                    <table class="gx-table">
                        <thead><tr><th>"#"</th><th>"kind"</th><th>"ok"</th><th>"late wakes"</th><th>"max late"</th><th>"length"</th></tr></thead>
                        <tbody>
                            {m.iterations.iter().map(|it| view! {
                                <tr data-warmup=it.measured.is_none().then_some("")>
                                    <td>{it.index}</td>
                                    <td>{if it.measured.is_some() { "measured" } else { "warmup" }}</td>
                                    <td title=it.error.clone().unwrap_or_default()>{if it.ok { "yes" } else { "no" }}</td>
                                    <td>{it.late_wakes}</td>
                                    <td>{format!("{} ms", short(it.max_lateness_ms))}</td>
                                    <td>{format!("{} ms", short(it.duration_ms))}</td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                    <details class="gx-drawer__params">
                        <summary>"parameters"</summary>
                        <dl class="gx-kv">
                            {m.params.iter().map(|(k, v)| view! { <dt>{k.clone()}</dt><dd>{v.clone()}</dd> }).collect_view()}
                        </dl>
                    </details>
                </section>
            </div>
            {latency.then(|| view! { <LatencyCharts cell=cell measurement=name.clone()/> })}
            {frames.then(|| view! { <FrameCharts cell=cell measurement=name.clone()/> })}
            {resize.then(|| view! { <ResizeCharts cell=cell measurement=name.clone()/> })}
            <PhaseChart cell=cell measurement=name.clone()/>
        </div>
    }
}
