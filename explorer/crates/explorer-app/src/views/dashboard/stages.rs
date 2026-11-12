//! Where the time of one interaction goes: per framework and variant, the median of each stage
//! from the input to the photon, stacked. The app owns the first stage; the GPU, the compositor
//! and the display own the rest.

use explorer_core::api::{CellInfo, MetricRef};
use explorer_core::units::short;
use leptos::prelude::*;

use crate::chart::{Axis, Bar, ChartCard, Rule, XyModel, XyPlot};
use crate::state::use_explorer;
use crate::views::kit::{Build, Data, Scope};

/// The stages: metric, name, color.
const STAGES: [(&str, &str, &str); 4] = [
    (
        "input_to_commit_p50",
        "app: input to commit",
        "var(--seq-4)",
    ),
    (
        "commit_to_ready_p50",
        "GPU: commit to ready",
        "var(--seq-3)",
    ),
    (
        "ready_to_latch_p50",
        "wait for the refresh: ready to latch",
        "var(--seq-2)",
    ),
    (
        "latch_to_present_p50",
        "scanout: latch to present",
        "var(--seq-1)",
    ),
];

/// Returns the stage metrics of every interaction.
pub fn metrics(interactions: &[&str]) -> Vec<MetricRef> {
    interactions
        .iter()
        .flat_map(|m| {
            STAGES
                .iter()
                .map(move |(s, ..)| MetricRef::new(m, "latency", s))
        })
        .collect()
}

/// The stage card of an interaction.
#[component]
pub fn StagesCard(data: Data, interaction: Signal<&'static str>, scope: Scope) -> impl IntoView {
    view! {
        <ChartCard
            title="Where the time goes"
            note="Median of each stage over the repeats, stacked · the medians need not add up to the median total · rule: one refresh"
            wide=true
        >
            <div class="gx-classes">
                {STAGES.iter().map(|(_, name, color)| view! {
                    <span><i class="gx-swatch" style:background=*color></i>{*name}</span>
                }).collect_view()}
            </div>
            <StagesPlot data=data interaction=interaction scope=scope/>
        </ChartCard>
    }
}

#[component]
fn StagesPlot(data: Data, interaction: Signal<&'static str>, scope: Scope) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, crate::state::Uncertainty::None);
        let m = interaction.get();
        let cells: Vec<&CellInfo> = b
            .cells("dashboard")
            .into_iter()
            .filter(|c| scope.holds(c.key.rows, c.key.updates, c.key.hz))
            .filter(|c| !prefs.valid_only || c.is_valid(m))
            .collect();
        let labels = cells
            .iter()
            .map(|c| (c.key.label(), crate::palette::ink(&c.key.framework)))
            .collect();
        let mut model = XyModel::new(
            Axis::value("milliseconds", "ms").valued(),
            Axis::categories(labels),
        );
        model.y.value = false;
        model.height = 38.0 + 24.0 * cells.len().max(1) as f64 + 16.0;
        if let Some(hz) = scope.hz.get() {
            model.rules.push(Rule {
                horizontal: false,
                value: 1000.0 / hz,
                label: "one refresh".to_owned(),
            });
        }
        for (i, c) in cells.iter().enumerate() {
            let mut x = 0.0;
            let total = data
                .stat(&MetricRef::new(m, "latency", "total_ms"), c.id)
                .map(|s| s.median);
            for (metric, name, color) in STAGES {
                let Some(v) = data
                    .stat(&MetricRef::new(m, "latency", metric), c.id)
                    .map(|s| s.median)
                else {
                    continue;
                };
                if v <= 0.0 {
                    continue;
                }
                let mut tip = vec![
                    ("framework".to_owned(), c.key.label()),
                    ("stage".to_owned(), name.to_owned()),
                    ("median".to_owned(), format!("{} ms", short(v))),
                ];
                if let Some(t) = total {
                    tip.push(("total, median".to_owned(), format!("{} ms", short(t))));
                }
                model.bars.push(Bar {
                    x0: x,
                    x1: x + v,
                    y0: i as f64 - 0.34,
                    y1: i as f64 + 0.34,
                    color: color.to_owned(),
                    group: c.key.framework.clone(),
                    tip,
                    cell: Some(c.id),
                });
                x += v;
            }
        }
        model
    });
    view! { <XyPlot model=model/> }
}
