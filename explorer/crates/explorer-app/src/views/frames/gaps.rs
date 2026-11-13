//! The gaps of one size and rate, from the frame profiles of its cells: a lane per iteration with
//! every late frame on its time axis, and the share of time (or frames) per class of lateness.

use std::sync::Arc;

use explorer_core::api::{CellInfo, FrameProfile, RunOverview};
use explorer_core::stats::{GAP_CLASSES, GapBreakdown, gap_breakdown};
use explorer_core::units::short;
use leptos::prelude::*;

use super::source::Source;
use crate::api::frame_profiles;
use crate::chart::{
    Axis, Bar, ChartCard, Lane, LaneGap, LanePlot, LaneRow, LanesModel, XyModel, XyPlot,
};
use crate::state::{PrefsSnapshot, cells_of, use_explorer};
use crate::views::kit::Scope;
use crate::widgets::{ChartSkeleton, Empty, ErrorCard, Segmented};

/// The gap cards of a source at the scope of the view.
#[component]
pub fn GapCards(overview: Arc<RunOverview>, source: Source, scope: Scope) -> impl IntoView {
    let run = overview.summary.id.clone();
    let version = overview.summary.modified_unix.unwrap_or(0);
    let profiles = Resource::new(
        move || (run.clone(), version, scope.size.get(), scope.hz.get()),
        move |(run, _, size, hz)| async move {
            let (Some((rows, _)), Some(hz)) = (size, hz) else {
                return Ok(Vec::new());
            };
            frame_profiles(
                run,
                source.scenario.to_owned(),
                source.measurement.to_owned(),
                hz,
                rows,
            )
            .await
        },
    );
    let fallback = || view! { <ChartSkeleton height=360/><ChartSkeleton height=360/> };
    view! {
        <Transition fallback=fallback>
            {move || {
                let overview = overview.clone();
                Suspend::new(async move {
                    match profiles.await {
                        Ok(list) if list.is_empty() => view! {
                            <Empty message="No frame records at this size and rate."/>
                        }.into_any(),
                        Ok(list) => {
                            let list = Arc::new(list);
                            view! {
                                <LanesCard overview=overview.clone() source=source profiles=list.clone()/>
                                <ClassesCard overview=overview profiles=list source=source/>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <ErrorCard
                                title="The frame records do not load"
                                error=e.to_string()
                                retry=Callback::new(move |_| profiles.refetch())
                            />
                        }.into_any(),
                    }
                })
            }}
        </Transition>
    }
}

/// Returns true when a breakdown holds frame time to share.
fn timed(b: &GapBreakdown) -> bool {
    b.time_ms.iter().sum::<f64>() > 0.0
}

/// Returns the shown cells with their profiles, in plot order.
fn shown<'a>(
    overview: &'a RunOverview,
    profiles: &'a [FrameProfile],
    source: &Source,
    prefs: &PrefsSnapshot,
) -> Vec<(&'a CellInfo, &'a FrameProfile)> {
    cells_of(overview, prefs, source.scenario)
        .into_iter()
        .filter(|c| !prefs.valid_only || c.is_valid(source.measurement))
        .filter_map(|c| Some((c, profiles.iter().find(|p| p.cell == c.id)?)))
        .collect()
}

#[component]
fn LanesCard(
    overview: Arc<RunOverview>,
    source: Source,
    profiles: Arc<Vec<FrameProfile>>,
) -> impl IntoView {
    let explorer = use_explorer();
    let (iters, set_iters) = signal("all".to_owned());
    let complete = profiles.iter().all(|p| p.lanes.iter().all(|l| l.complete));
    let note = if complete {
        "One lane per iteration · light: first to last present · bars: late frames over their interval, darker misses more refreshes · outline: content stalled · tick: double step"
    } else {
        "One lane per iteration · bars: frames over 1.5× the median interval, darker misses more refreshes · the report lists no other frames, so a lane whose median frame is late takes its color"
    };
    let controls = ViewFn::from(move || {
        view! {
            <Segmented
                options=vec![("all".to_owned(), "all iterations".to_owned()), ("one".to_owned(), "first".to_owned())]
                value=Signal::derive(move || iters.get())
                on_change=Callback::new(move |v| set_iters.set(v))
            />
        }
    });
    let model = Signal::derive(move || {
        let one = iters.get() == "one";
        let prefs = explorer.prefs.snapshot();
        let rows = shown(&overview, &profiles, &source, &prefs)
            .into_iter()
            .map(|(c, p)| LaneRow {
                key: c.id.to_string(),
                label: c.key.framework.clone(),
                group: c.key.framework.clone(),
                ink: crate::palette::ink(&c.key.framework),
                cell: Some(c.id),
                lanes: p
                    .lanes
                    .iter()
                    .take(if one { 1 } else { usize::MAX })
                    .map(|l| Lane {
                        base_missed: if l.frames < 2 {
                            Some(10)
                        } else {
                            (!l.complete)
                                .then(|| {
                                    ((l.threshold_ms / 1.5 / p.refresh_ms).round() as u32)
                                        .saturating_sub(1)
                                })
                                .filter(|m| *m > 0)
                        },
                        index: l.index,
                        window_ms: l.window_ms,
                        presents: l.presents,
                        frames: l.frames,
                        complete: l.complete,
                        threshold_ms: l.threshold_ms,
                        gaps: l
                            .gaps
                            .iter()
                            .map(|g| LaneGap {
                                t_ms: g.t_ms,
                                interval_ms: g.interval_ms,
                                missed: g.missed,
                                stalled: g.stalled,
                                double_step: g.double_step,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect();
        LanesModel {
            rows,
            refresh_ms: profiles.first().map_or(16.7, |p| p.refresh_ms),
        }
    });
    view! {
        <ChartCard title="Late frames over time" note=note controls=controls wide=true>
            <ClassLegend/>
            <LanePlot model=model/>
        </ChartCard>
    }
}

/// The swatches of the lateness classes.
#[component]
fn ClassLegend() -> impl IntoView {
    view! {
        <div class="gx-classes">
            {GAP_CLASSES.iter().enumerate().map(|(i, c)| view! {
                <span><i class="gx-swatch" style:background=format!("var(--late-{i})")></i>{c.name}</span>
            }).collect_view()}
        </div>
    }
}

#[component]
fn ClassesCard(
    overview: Arc<RunOverview>,
    profiles: Arc<Vec<FrameProfile>>,
    source: Source,
) -> impl IntoView {
    let explorer = use_explorer();
    let (by, set_by) = signal("time".to_owned());
    let controls = ViewFn::from(move || {
        view! {
            <Segmented
                options=vec![("time".to_owned(), "time".to_owned()), ("frames".to_owned(), "frames".to_owned())]
                value=Signal::derive(move || by.get())
                on_change=Callback::new(move |v| set_by.set(v))
            />
        }
    });
    let ov = overview.clone();
    let pr = profiles.clone();
    let left_out = Signal::derive(move || {
        shown(&ov, &pr, &source, &explorer.prefs.snapshot())
            .iter()
            .filter(|(_, p)| gap_breakdown(&p.lanes).is_none_or(|b| !timed(&b)))
            .map(|(c, _)| c.key.framework.clone())
            .collect::<Vec<_>>()
    });
    let note = Signal::derive(move || {
        let what = if by.get() == "time" {
            "Share of the frame time in each class: a late frame counts its whole interval"
        } else {
            "Share of the presented frames in each class"
        };
        let out = left_out.get();
        if out.is_empty() {
            format!("{what} · over every iteration")
        } else {
            format!(
                "{what} · left out, as their reports list only frames over 1.5× a median above one refresh, or no frames: {}",
                out.join(", ")
            )
        }
    });
    let model = Signal::derive(move || {
        let time = by.get() == "time";
        let prefs = explorer.prefs.snapshot();
        let rows: Vec<_> = shown(&overview, &profiles, &source, &prefs)
            .into_iter()
            .filter_map(|(c, p)| Some((c, gap_breakdown(&p.lanes)?)))
            .filter(|(_, b)| timed(b))
            .collect();
        let labels = rows
            .iter()
            .map(|(c, _)| {
                (
                    c.key.framework.clone(),
                    crate::palette::ink(&c.key.framework),
                )
            })
            .collect();
        let mut model = XyModel::new(
            Axis::value(
                if time {
                    "share of the frame time"
                } else {
                    "share of the frames"
                },
                "%",
            )
            .valued(),
            Axis::categories(labels),
        );
        model.x.domain = Some((0.0, 100.0));
        model.x.value = false;
        model.y.value = false;
        model.height = 38.0 + 24.0 * rows.len().max(1) as f64 + 16.0;
        for (i, (c, b)) in rows.iter().enumerate() {
            let shares = if time {
                b.time_shares()
            } else {
                b.frame_shares()
            };
            let mut x = 0.0;
            for (k, share) in shares.iter().enumerate() {
                let w = share * 100.0;
                if w < 0.01 {
                    continue;
                }
                model.bars.push(Bar {
                    x0: x,
                    x1: x + w,
                    y0: i as f64 - 0.34,
                    y1: i as f64 + 0.34,
                    color: format!("var(--late-{k})"),
                    group: c.key.framework.clone(),
                    tip: vec![
                        ("framework".to_owned(), c.key.framework.clone()),
                        ("class".to_owned(), GAP_CLASSES[k].name.to_owned()),
                        ("share".to_owned(), format!("{}%", short(w))),
                        ("frames".to_owned(), b.frames[k].to_string()),
                        ("time".to_owned(), format!("{} ms", short(b.time_ms[k]))),
                    ],
                    cell: Some(c.id),
                });
                x += w;
            }
        }
        model
    });
    view! {
        <ChartCard title="Where the frame time goes" note=note controls=controls wide=true>
            <ClassLegend/>
            <XyPlot model=model/>
        </ChartCard>
    }
}
