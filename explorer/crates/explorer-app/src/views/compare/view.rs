//! The layout of the compare view and the pairing of cells: two frameworks inside the run, or
//! the same cells in the run and the compare run.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use explorer_core::story::PACED;
use leptos::prelude::*;

use super::forest::{Pair, Verdict, ratio, row, verdict};
use crate::api::metric_catalog;
use crate::chart::{Axis, ChartCard, DotModel, DotPlot, DotRow, Rule};
use crate::palette;
use crate::state::{RunView, use_explorer, use_param};
use crate::views::kit::{Data, ViewHead, WithData, missed, response_metrics, slowest};
use crate::widgets::{ChartSkeleton, Empty, Segmented, SelectBox};

/// The key metrics beside the responses and the pacing measurements of the story.
const KEY: [(&str, &str, &str); 9] = [
    ("startup", "startup", "content_ms"),
    ("idle", "resources", "pss_max"),
    ("idle", "resources", "gpu_proc_mem_max"),
    ("idle", "resources", "cpu_ms_per_s"),
    ("idle", "resources", "wakeups_per_s"),
    ("resize", "resize", "configure_to_present_p50"),
    ("resize", "resize", "mismatched_frames"),
    ("live", "resources", "cpu_ms_per_s"),
    ("live", "frame_cost", "gpu_fence_ms_per_s"),
];

fn key_metrics() -> Vec<MetricRef> {
    let mut out = response_metrics();
    for step in PACED {
        out.push(slowest(step.name));
        out.push(missed(step.name));
    }
    out.extend(KEY.iter().map(|(a, b, c)| MetricRef::new(a, b, c)));
    out
}

/// The compare view.
#[component]
pub fn CompareView() -> impl IntoView {
    let explorer = use_explorer();
    let (a, _) = use_param("a", "");
    let (b, _) = use_param("b", "");
    let (mode, set_mode) = use_param("mode", "frameworks");
    let (scope, set_scope) = use_param("scope", "key");
    let (sort, set_sort) = use_param("sort", "matrix");
    view! {
        <div class="gx-view">
            <ViewHead
                title="Compare"
                text="Every matching cell of A against B: how many times better the better side is, from the medians, with the 95% bootstrap interval over repeats or iterations. Left of 1×: A is better (blue) · right: B is better (orange) · gray hollow: the interval holds 1×. Single values have no interval; they count as different beyond 1.1×."
            >
                <Segmented
                    options=vec![("frameworks".to_owned(), "frameworks".to_owned()), ("runs".to_owned(), "runs".to_owned())]
                    value=Signal::derive(move || mode.get())
                    on_change=set_mode
                />
                <Segmented
                    options=vec![("key".to_owned(), "key metrics".to_owned()), ("all".to_owned(), "every metric".to_owned())]
                    value=Signal::derive(move || scope.get())
                    on_change=set_scope
                />
                <Segmented
                    options=vec![("matrix".to_owned(), "by scenario".to_owned()), ("effect".to_owned(), "largest first".to_owned())]
                    value=Signal::derive(move || sort.get())
                    on_change=set_sort
                />
            </ViewHead>
            <RunView view={move |o: Arc<RunOverview>| {
                let run = o.summary.id.clone();
                let mut fws = o.frameworks();
                palette::sort(&mut fws);
                let first = fws.first().cloned().unwrap_or_default();
                let second = fws.get(1).cloned().unwrap_or_default();
                let a_spec = Memo::new(move |_| Some(a.get()).filter(|s| !s.is_empty()).unwrap_or_else(|| first.clone()));
                let b_spec = Memo::new(move |_| Some(b.get()).filter(|s| !s.is_empty()).unwrap_or_else(|| second.clone()));
                // Framework mode reads the two sides only; run mode reads every framework.
                let only = Signal::derive(move || {
                    if mode.get() == "runs" {
                        return Vec::new();
                    }
                    let mut out = vec![a_spec.get(), b_spec.get()];
                    out.dedup();
                    out
                });
                // The pickers sit outside the data below, so a reload never disposes them.
                let needs_run = move || {
                    mode.try_get().as_deref() == Some("runs") && explorer.prefs.compare.try_get().is_some_and(|c| c.is_none())
                };
                let o_pick = o.clone();
                let o_sides = o.clone();
                let list = Resource::new(
                    move || (run.clone(), scope.get() == "all"),
                    |(run, all)| async move {
                        if all {
                            metric_catalog(run).await.map(|c| c.into_iter().map(|m| m.metric).collect::<Vec<_>>())
                        } else {
                            Ok(key_metrics())
                        }
                    },
                );
                view! {
                    {move || (mode.try_get().as_deref() == Some("frameworks")).then(|| view! {
                        <Sides overview=o_sides.clone() a_spec=a_spec b_spec=b_spec/>
                    })}
                    {move || needs_run().then(|| view! { <RunPick overview=o_pick.clone()/> })}
                    <Transition fallback=|| view! { <ChartSkeleton/> }>
                        {move || {
                            let o = o.clone();
                            Suspend::new(async move {
                                let metrics = Signal::stored(list.await.unwrap_or_else(|_| key_metrics()));
                                view! {
                                    <WithData overview=o metrics=metrics frameworks=only skeletons=1 view=move |data| {
                                        if mode.get() == "runs" && data.compare.is_none() {
                                            return ().into_any();
                                        }
                                        view! { <Forest data=data mode=mode metrics=metrics sort=sort a_spec=a_spec b_spec=b_spec/> }.into_any()
                                    }/>
                                }
                            })
                        }}
                    </Transition>
                }
            }}/>
        </div>
    }
}

/// The pickers of the two sides: every framework.
#[component]
fn Sides(overview: Arc<RunOverview>, a_spec: Memo<String>, b_spec: Memo<String>) -> impl IntoView {
    let explorer = use_explorer();
    let mut fws = overview.frameworks();
    palette::sort(&mut fws);
    let sides: Vec<(String, String)> = fws.into_iter().map(|f| (f.clone(), f)).collect();
    let set =
        move |key: &'static str| Callback::new(move |v: String| explorer.query.set(key, Some(v)));
    view! {
        <div class="gx-view__controls">
            <SelectBox label="A" options=sides.clone() value=Signal::derive(move || a_spec.get()) on_change=set("a")/>
            <button type="button" class="gx-btn" title="Swap A and B" on:click=move |_| {
                let (x, y) = (a_spec.get_untracked(), b_spec.get_untracked());
                explorer.query.set_all(vec![("a", Some(y)), ("b", Some(x))]);
            }>"⇄"</button>
            <SelectBox label="B" options=sides value=Signal::derive(move || b_spec.get()) on_change=set("b")/>
        </div>
    }
}

#[component]
fn RunPick(overview: Arc<RunOverview>) -> impl IntoView {
    let explorer = use_explorer();
    let me = overview.summary.id.clone();
    view! {
        <Empty message="Pick a run to compare with: the same cells of both runs pair up."/>
        <Suspense fallback=|| ()>
            {move || {
                let me = me.clone();
                Suspend::new(async move {
                    let runs = explorer.runs.await.unwrap_or_default();
                    let options: Vec<(String, String)> = std::iter::once((String::new(), "choose a run…".to_owned()))
                        .chain(runs.into_iter().filter(|r| r.readable() && r.id != me).map(|r| (r.id.clone(), r.id)))
                        .collect();
                    view! {
                        <div class="gx-view__controls">
                            <SelectBox options=options value=String::new() on_change=Callback::new(move |v: String| explorer.query.set("cmp", Some(v)))/>
                        </div>
                    }
                })
            }}
        </Suspense>
    }
}

#[component]
fn Forest(
    data: Data,
    mode: Memo<String>,
    metrics: Signal<Vec<MetricRef>>,
    sort: Memo<String>,
    a_spec: Memo<String>,
    b_spec: Memo<String>,
) -> impl IntoView {
    let explorer = use_explorer();
    let summary = RwSignal::new((0usize, 0usize, 0usize));
    let d = data.clone();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let runs = mode.get() == "runs";
        let overview = d.overview();
        let (afw, bfw) = (a_spec.get(), b_spec.get());
        let (a_label, b_label) = if runs {
            (
                overview.summary.id.clone(),
                d.compare
                    .as_ref()
                    .map(|c| c.overview.summary.id.clone())
                    .unwrap_or_default(),
            )
        } else {
            (afw.clone(), bfw.clone())
        };
        let mut pairs: Vec<Pair> = Vec::new();
        for m in metrics.get() {
            let Some(col) = d.run.col(&m) else { continue };
            let unit = explorer_core::units::display(&col.unit).label.to_owned();
            let factor = explorer_core::units::display(&col.unit).factor;
            for (i, cell_id) in col.cells.iter().enumerate() {
                // A live run can hold cells newer than the overview of the page.
                let Some(a_cell) = overview.cells.get(*cell_id as usize) else {
                    continue;
                };
                if !prefs.shows_rate(a_cell.key.hz)
                    || (prefs.valid_only && !a_cell.is_valid(&m.measurement))
                {
                    continue;
                }
                let sb = if runs {
                    if !prefs.shows_framework(&a_cell.key.framework) {
                        continue;
                    }
                    d.ghost(&m, &a_cell.key).cloned()
                } else {
                    if a_cell.key.framework != afw {
                        continue;
                    }
                    overview
                        .cells
                        .iter()
                        .find(|c| c.key.framework == bfw && c.key.same_point(&a_cell.key))
                        .and_then(|c| d.stat(&m, c.id).cloned())
                };
                let Some(sb) = sb else { continue };
                pairs.push(Pair {
                    metric: m.clone(),
                    better: col.better,
                    unit: unit.clone(),
                    a: a_cell,
                    sa: col.stats[i].scaled(factor),
                    sb: sb.scaled(factor),
                });
            }
        }
        let mut rated: Vec<(Pair, explorer_core::stats::Ratio)> = pairs
            .into_iter()
            .filter_map(|p| ratio(&p).map(|r| (p, r)))
            .collect();
        let counts = rated
            .iter()
            .fold((0, 0, 0), |(a, b, s), (p, r)| match verdict(r, p.better) {
                Verdict::ABetter => (a + 1, b, s),
                Verdict::BBetter => (a, b + 1, s),
                Verdict::Same => (a, b, s + 1),
            });
        summary.set(counts);
        let mut rows: Vec<DotRow> = Vec::new();
        if sort.get() == "effect" {
            rated.sort_by(|x, y| y.1.estimate.ln().abs().total_cmp(&x.1.estimate.ln().abs()));
            rows.extend(rated.iter().map(|(p, r)| row(p, *r, &a_label, &b_label)));
        } else {
            let head_of = |p: &Pair| {
                if runs {
                    p.a.key.framework.clone()
                } else {
                    p.a.key.scenario.clone()
                }
            };
            let mut heads: Vec<String> = Vec::new();
            for (p, _) in &rated {
                let head = head_of(p);
                if !heads.contains(&head) {
                    heads.push(head);
                }
            }
            rated.sort_by_key(|(p, _)| heads.iter().position(|h| *h == head_of(p)));
            let mut last = String::new();
            for (p, r) in &rated {
                let head = head_of(p);
                if head != last {
                    rows.push(DotRow::header(head.clone()));
                    last = head;
                }
                let mut rw = row(p, *r, &a_label, &b_label);
                if runs {
                    rw.label = format!("{} · {}", p.a.key.framework, rw.label);
                }
                rows.push(rw);
            }
        }
        let mut x = Axis::value(format!("← {a_label} better · {b_label} better →"), "×");
        x.kind = explorer_core::scale::ScaleKind::Log;
        x.zero = false;
        DotModel {
            rows,
            x,
            rules: vec![Rule {
                horizontal: false,
                value: 1.0,
                label: String::new(),
            }],
            hide_labels: false,
            factors: true,
        }
    });
    view! {
        <ChartCard
            title="A against B"
            note=Signal::derive(move || {
                let (a, b, s) = summary.get();
                format!("A better in {a} · B better in {b} · no clear difference in {s} · click a row to open the A cell")
            })
            wide=true
        >
            <DotPlot model=model/>
        </ChartCard>
    }
}
