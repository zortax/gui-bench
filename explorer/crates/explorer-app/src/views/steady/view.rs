//! The layout of the steady view: the scenario, size and rate controls, the frame-time profile
//! and one dot plot per cost metric.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use explorer_core::scale::ScaleKind;
use leptos::prelude::*;

use super::efficiency::Efficiency;
use crate::chart::{Axis, ChartCard, DotModel, DotPlot, Rule, spread_note, use_card};
use crate::state::{RunView, Selection, use_explorer, use_param};
use crate::views::frames::{ProfileCard, SOURCES, Source};
use crate::views::kit::{
    Build, Data, Scope, ScopeBar, ViewHead, WithData, baseline_label, use_scope,
};
use crate::widgets::{Empty, Segmented};

/// The cost metrics of the pacing measurement: section, metric, title, axis words.
const COSTS: [(&str, &str, &str, &str); 8] = [
    (
        "resources",
        "cpu_ms_per_s",
        "App CPU time",
        "CPU ms per second",
    ),
    (
        "frame_cost",
        "gpu_fence_ms_per_s",
        "GPU time",
        "commit to buffer fence, ms per second",
    ),
    (
        "frame_pacing",
        "stale_refreshes",
        "Stale refreshes",
        "% of refreshes that repeat the last frame",
    ),
    (
        "frame_cost",
        "cpu_ms_per_frame",
        "CPU per frame",
        "CPU ms per frame",
    ),
    (
        "frame_cost",
        "damage_fraction_mean",
        "Damaged area",
        "fraction of the window per frame",
    ),
    (
        "resources",
        "wakeups_per_s",
        "Wakeups",
        "context switches per second",
    ),
    (
        "frame_pacing",
        "interval_max",
        "Longest gap",
        "ms between two presents, worst",
    ),
    (
        "resources",
        "gpu_proc_busy_percent",
        "GPU busy (driver)",
        "percent, per process",
    ),
];

fn metrics() -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = COSTS
        .iter()
        .map(|(s, m, _, _)| MetricRef::new("pacing", s, m))
        .collect();
    out.extend(super::efficiency::metrics());
    out.extend(crate::views::frames::metrics(&pacing_source("ticker")));
    out
}

/// Returns the frame source of a steady scenario.
fn pacing_source(scenario: &str) -> Source {
    SOURCES
        .iter()
        .copied()
        .find(|s| s.scenario == scenario)
        .unwrap_or(SOURCES[0])
}

/// The steady view.
#[component]
pub fn SteadyView() -> impl IntoView {
    let explorer = use_explorer();
    let (scenario, set_scenario) = use_param("sc", "ticker");
    view! {
        <div class="gx-view">
            <ViewHead
                title="Ticker & spinner"
                text="Continuous animation: a timer updates table rows every 16 ms (ticker), or a 24 px square turns above static rows (spinner). Lower cost per second is better at equal frame times."
            >
                <Segmented
                    options=vec![("ticker".to_owned(), "ticker".to_owned()), ("spinner".to_owned(), "spinner".to_owned())]
                    value=Signal::derive(move || scenario.get())
                    on_change=set_scenario
                />
            </ViewHead>
            <RunView view={move |o: Arc<RunOverview>| {
                let o2 = o.clone();
                view! {
                    <WithData overview=o metrics=metrics() skeletons=4 view=move |data| {
                        let sc = scenario.get();
                        let prefs = explorer.prefs.snapshot();
                        if Selection::new(&o2, &prefs, &sc).cells.is_empty() {
                            return view! {
                                <Empty message=format!("No {sc} cells shown: the run has none, or the filters hide them.")/>
                                <Efficiency data=data/>
                            }.into_any();
                        }
                        view! { <Costs data=data scenario=scenario/> }.into_any()
                    }/>
                }
            }}/>
        </div>
    }
}

#[component]
fn Costs(data: Data, scenario: Memo<String>) -> impl IntoView {
    let scope = use_scope(
        data.run.overview.clone(),
        Signal::derive(move || scenario.get()),
    );
    view! {
        <div class="gx-view__controls"><ScopeBar scope=scope/></div>
        <div class="gx-grid gx-grid--wide">
            {
                let data = data.clone();
                move || view! {
                    <ProfileCard data=data.clone() source=pacing_source(&scenario.get()) scope=scope per_refresh=Signal::stored(false)/>
                }
            }
        </div>
        <div class="gx-grid">
            {COSTS.iter().map(|(section, metric, title, words)| {
                let m = MetricRef::new("pacing", section, metric);
                let d = data.clone();
                view! {
                    <ChartCard title=*title>
                        <CostPlot data=d scenario=scenario metric=m words=*words scope=scope/>
                    </ChartCard>
                }
            }).collect_view()}
        </div>
        <Efficiency data=data/>
    }
}

#[component]
fn CostPlot(
    data: Data,
    scenario: Memo<String>,
    metric: MetricRef,
    words: &'static str,
    scope: Scope,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    let samples = data.samples(&metric);
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note.set(spread_note(samples, ctx.uncertainty(), "Dot"));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let hz = scope.hz.get().unwrap_or(60.0);
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let rows = b.dot_rows(&scenario.get(), &metric, |k| {
            scope.holds(k.rows, k.updates, k.hz)
        });
        let base = prefs.base.is_some();
        let mut x = Axis::value(
            if base {
                baseline_label(&prefs, "")
            } else {
                words.to_owned()
            },
            "",
        );
        if base {
            x.kind = ScaleKind::Log;
        }
        let rules = if metric.metric.starts_with("interval") && !base {
            vec![Rule {
                horizontal: false,
                value: 1000.0 / hz,
                label: "one refresh".to_owned(),
            }]
        } else {
            Vec::new()
        };
        DotModel {
            rows,
            x,
            rules,
            hide_labels: false,
            factors: false,
        }
    });
    view! { <DotPlot model=model/> }
}
