//! The layout of the steady view: the scenario, size and rate controls and one dot plot per
//! cost metric.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use explorer_core::scale::ScaleKind;
use leptos::prelude::*;

use super::efficiency::Efficiency;
use crate::chart::{Axis, ChartCard, DotModel, DotPlot, Rule, spread_note, use_card};
use crate::state::{RunView, Selection, use_explorer, use_param};
use crate::views::kit::{Build, Data, SizePicker, ViewHead, WithData, baseline_label, sizes_label};
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
        "fps",
        "Delivered frames",
        "frames per second",
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
        "jank_count",
        "Janky frames",
        "frames 1.5× slower than the median",
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
    out
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
                text="Continuous animation: a timer updates table rows every 16 ms (ticker), or a 24 px square turns above static rows (spinner). Lower cost per second is better at equal delivered frames."
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
    let explorer = use_explorer();
    let size = RwSignal::new(String::new());
    let rate = RwSignal::new(String::new());
    let d = data.clone();
    let sizes = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        Selection::new(d.overview(), &prefs, &scenario.get())
            .sizes()
            .into_iter()
            .map(|(r, u)| (format!("{r}:{}", u.unwrap_or(0)), sizes_label(r, u)))
            .collect::<Vec<_>>()
    });
    let d = data.clone();
    let rates = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        Selection::new(d.overview(), &prefs, &scenario.get())
            .rates()
            .into_iter()
            .map(|hz| (hz.to_string(), format!("{hz} Hz")))
            .collect::<Vec<_>>()
    });
    view! {
        <div class="gx-view__controls">
            <SizePicker options=sizes value=size title="Rows and updates per tick"/>
            <SizePicker options=rates value=rate title="Refresh rate"/>
        </div>
        <div class="gx-grid">
            {COSTS.iter().map(|(section, metric, title, words)| {
                let m = MetricRef::new("pacing", section, metric);
                let d = data.clone();
                view! {
                    <ChartCard title=*title>
                        <CostPlot data=d scenario=scenario metric=m words=*words size=size rate=rate/>
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
    size: RwSignal<String>,
    rate: RwSignal<String>,
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
        let (rows, updates) = size.get().split_once(':').map_or((0, 0), |(r, u)| {
            (r.parse().unwrap_or(0), u.parse().unwrap_or(0))
        });
        let hz: f64 = rate.get().parse().unwrap_or(0.0);
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let rows = b.dot_rows(&scenario.get(), &metric, |k| {
            k.rows == rows && k.updates.unwrap_or(0) == updates && k.hz == hz
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
        let rules = if metric.metric == "fps" && !base {
            vec![Rule {
                horizontal: false,
                value: hz,
                label: String::new(),
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
