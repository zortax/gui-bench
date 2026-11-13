//! A lane plot: one row per framework with one thin lane per iteration on the time
//! axis of the iteration. A lane shows its window, the span between its first and last present,
//! and every late frame as a bar over its interval, colored by the refreshes it missed. A lane that
//! lists only its slowest frames takes the color of its typical frame. Stalled frames show as
//! outlines, double steps as ticks. The time axis zooms and pans.

use explorer_core::api::CellId;
use explorer_core::scale::{Scale, ScaleKind};
use explorer_core::stats::{GAP_CLASSES, gap_class};
use explorer_core::units::{short, thousands};
use leptos::html;
use leptos::prelude::*;

use super::card::use_card;
use super::interact::{Drag, local, on_zoom_wheel};
use super::size::use_width;
use crate::state::use_explorer;

/// A late, stalled or double-stepped frame on a lane.
#[derive(Debug, Clone, PartialEq)]
pub struct LaneGap {
    /// The present, in ms from the start of the iteration.
    pub t_ms: f64,
    pub interval_ms: f64,
    pub missed: u32,
    pub stalled: bool,
    pub double_step: bool,
}

/// One iteration.
#[derive(Debug, Clone, PartialEq)]
pub struct Lane {
    pub index: u32,
    pub window_ms: f64,
    pub presents: Option<(f64, f64)>,
    pub frames: u32,
    /// The lane lists every late frame.
    pub complete: bool,
    pub threshold_ms: f64,
    /// The refreshes a typical frame misses, when the lane lists only the slowest frames.
    pub base_missed: Option<u32>,
    pub gaps: Vec<LaneGap>,
}

/// The lanes of one framework.
#[derive(Debug, Clone, PartialEq)]
pub struct LaneRow {
    pub key: String,
    pub label: String,
    pub group: String,
    pub ink: String,
    pub cell: Option<CellId>,
    pub lanes: Vec<Lane>,
}

/// A lane plot.
#[derive(Debug, Clone, PartialEq)]
pub struct LanesModel {
    pub rows: Vec<LaneRow>,
    /// One refresh, for the tooltip.
    pub refresh_ms: f64,
}

const LANE: f64 = 7.0;
const LANE_GAP: f64 = 2.0;
const ROW_PAD: f64 = 7.0;
const TOP: f64 = 4.0;
const AXIS: f64 = 38.0;

fn f(v: f64) -> String {
    format!("{v:.1}")
}

fn row_height(r: &LaneRow) -> f64 {
    let n = r.lanes.len().max(1) as f64;
    n * LANE + (n - 1.0) * LANE_GAP + 2.0 * ROW_PAD
}

/// Returns the top of every row and the bottom of the last.
fn row_tops(m: &LanesModel) -> (Vec<f64>, f64) {
    let mut y = TOP;
    let tops = m
        .rows
        .iter()
        .map(|r| {
            let top = y;
            y += row_height(r);
            top
        })
        .collect();
    (tops, y)
}

/// Returns the class fill of a frame.
fn fill(missed: u32) -> String {
    format!("var(--late-{})", gap_class(missed))
}

/// What the pointer is over: a row, its lane, and the gap nearest the pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Spot {
    row: usize,
    lane: Option<usize>,
    gap: Option<usize>,
}

/// A lane plot that fills the width of its card.
#[component]
pub fn LanePlot(#[prop(into)] model: Signal<LanesModel>) -> impl IntoView {
    let ctx = use_card();
    let explorer = use_explorer();
    let model = Memo::new(move |_| model.get());
    let wrap = NodeRef::<html::Div>::new();
    let width = use_width(wrap, ctx.width_hint);
    let zoom = RwSignal::new(None::<(f64, f64)>);
    let hover = RwSignal::new(None::<Spot>);
    let drag = RwSignal::new(None::<Drag>);
    Effect::new(move |_| {
        let m = model.read();
        let mut csv = String::from("row,iteration,t_ms,interval_ms,missed,stalled,double_step\n");
        for r in &m.rows {
            for l in &r.lanes {
                for g in &l.gaps {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        r.label, l.index, g.t_ms, g.interval_ms, g.missed, g.stalled, g.double_step
                    ));
                }
            }
        }
        ctx.csv.set(csv);
    });
    Effect::new(move |_| ctx.zoomed.set(zoom.get().is_some()));
    Effect::new(move |prev: Option<u32>| {
        let n = ctx.reset.get();
        if prev.is_some() {
            zoom.set(None);
        }
        n
    });
    let label_w = Memo::new(move |_| {
        model.with(|m| m.rows.iter().map(|r| r.label.len()).max().unwrap_or(4)) as f64 * 7.2 + 14.0
    });
    let domain = Memo::new(move |_| {
        zoom.get().unwrap_or_else(|| {
            let hi = model.with(|m| {
                m.rows
                    .iter()
                    .flat_map(|r| r.lanes.iter().map(|l| l.window_ms))
                    .fold(1.0, f64::max)
            });
            (0.0, hi)
        })
    });
    let scale_of = |w: f64, d: (f64, f64), l: f64| {
        Scale::new(ScaleKind::Linear, d, (l, (w - 16.0).max(l + 40.0)))
    };
    let scale = move || scale_of(width.get(), domain.get(), label_w.get());
    // Event handlers read the scale without tracking.
    let scale_now = move || {
        scale_of(
            width.get_untracked(),
            domain.get_untracked(),
            label_w.get_untracked(),
        )
    };
    let zoom_by = move |factor: f64, at: Option<f64>| {
        let s = scale_now();
        let (l, r) = (label_w.get_untracked(), width.get_untracked() - 16.0);
        let (lo, hi) = s.zoomed(at.unwrap_or((l + r) / 2.0), factor);
        zoom.set(Some((lo.max(-50.0), hi)));
    };
    Effect::new(move |prev: Option<u32>| {
        let n = ctx.zoom_in.get();
        if prev.is_some() {
            zoom_by(0.7, None);
        }
        n
    });
    Effect::new(move |prev: Option<u32>| {
        let n = ctx.zoom_out.get();
        if prev.is_some() {
            zoom_by(1.0 / 0.7, None);
        }
        n
    });
    on_zoom_wheel(wrap, move |ev: web_sys::WheelEvent| {
        if let Some(el) = wrap.get_untracked() {
            let (x, _) = local(&el, &ev);
            zoom_by(if ev.delta_y() < 0.0 { 0.8 } else { 1.0 / 0.8 }, Some(x));
        }
    });
    let spot_at = move |x: f64, y: f64| -> Option<Spot> {
        let s = scale_now();
        model.with_untracked(|m| {
            let (tops, _) = row_tops(m);
            let row = tops.iter().rposition(|t| y >= *t)?;
            let r = m.rows.get(row)?;
            if y > tops[row] + row_height(r) {
                return None;
            }
            let ly = y - tops[row] - ROW_PAD;
            let lane = ((ly + LANE_GAP / 2.0) / (LANE + LANE_GAP)).floor();
            let lane = (lane >= 0.0 && (lane as usize) < r.lanes.len()).then_some(lane as usize);
            let t = s.invert(x);
            let slack = (s.invert(x + 4.0) - t).abs();
            let gap = lane.and_then(|li| {
                r.lanes[li]
                    .gaps
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| t >= g.t_ms - g.interval_ms - slack && t <= g.t_ms + slack)
                    .min_by(|a, b| (a.1.t_ms - t).abs().total_cmp(&(b.1.t_ms - t).abs()))
                    .map(|(i, _)| i)
            });
            Some(Spot { row, lane, gap })
        })
    };
    let svg = move || {
        let w = width.get();
        if w < 40.0 {
            return None;
        }
        let s = scale();
        let left = label_w.get();
        let right = (w - 16.0).max(left + 40.0);
        let m = model.read();
        let (tops, bottom) = row_tops(&m);
        let h = bottom + AXIS;
        let hl = ctx.highlight;
        let x = |t: f64| s.map(t).clamp(left, right);
        let ticks = s
            .ticks(((right - left) / 80.0).round().max(2.0) as usize)
            .into_iter()
            .map(|t| {
                let px = f(s.map(t.value));
                view! {
                    <line class="gx-gridline" data-minor=(!t.major).then_some("") x1=px.clone() x2=px.clone() y1=f(TOP) y2=f(bottom)/>
                    {t.major.then(|| view! { <text class="gx-tick" x=px y=f(bottom + 16.0) text-anchor="middle">{thousands(t.value)}</text> })}
                }
            })
            .collect_view();
        let rows = m
            .rows
            .iter()
            .enumerate()
            .map(|(ri, r)| {
                let top = tops[ri];
                let group = r.group.clone();
                let dim = move || hl.with(|h| h.as_ref().is_some_and(|h| *h != group)).then_some("");
                let lanes = r
                    .lanes
                    .iter()
                    .enumerate()
                    .map(|(li, l)| {
                        let y = top + ROW_PAD + li as f64 * (LANE + LANE_GAP);
                        let span = l.presents.map(|(a, b)| view! {
                            <rect class="gx-lane__span" x=f(x(a)) y=f(y) width=f((x(b) - x(a)).max(0.5)) height=f(LANE)/>
                        });
                        let gaps = l
                            .gaps
                            .iter()
                            .filter(|g| g.t_ms >= s.bounds().0 - g.interval_ms && g.t_ms - g.interval_ms <= s.bounds().1)
                            .map(|g| {
                                let x0 = x(g.t_ms - g.interval_ms);
                                let x1 = x(g.t_ms);
                                if g.missed > 0 {
                                    view! { <rect class="gx-lane__gap" x=f(x0) y=f(y) width=f((x1 - x0).max(1.2)) height=f(LANE) style:fill=fill(g.missed)/> }.into_any()
                                } else if g.stalled {
                                    view! { <rect class="gx-lane__stall" x=f(x0) y=f(y + 0.5) width=f((x1 - x0).max(1.2)) height=f(LANE - 1.0)/> }.into_any()
                                } else {
                                    view! { <line class="gx-lane__double" x1=f(x1) x2=f(x1) y1=f(y - 1.5) y2=f(y + LANE + 1.5)/> }.into_any()
                                }
                            })
                            .collect_view();
                        let base = l.base_missed.map(|m| view! {
                            <rect class="gx-lane__base" x=f(x(0.0)) y=f(y) width=f((x(l.window_ms) - x(0.0)).max(0.0)) height=f(LANE) style:fill=fill(m)/>
                        });
                        view! {
                            <rect class="gx-lane__bg" x=f(x(0.0)) y=f(y) width=f((x(l.window_ms) - x(0.0)).max(0.0)) height=f(LANE)/>
                            {base}
                            {span}
                            {gaps}
                        }
                    })
                    .collect_view();
                let label_y = top + row_height(r) / 2.0;
                view! {
                    <g class="gx-row" data-dim=dim>
                        // A hairline parts the rows. A band behind every other row would sit between
                        // the tones of the lanes and hide the frames that are on time.
                        {(ri > 0).then(|| view! { <line class="gx-rowrule" x1="0" x2=f(right + 6.0) y1=f(top) y2=f(top)/> })}
                        <text class="gx-rowlabel" x=f(left - 8.0) y=f(label_y) text-anchor="end" dominant-baseline="middle" style:fill=r.ink.clone()>{r.label.clone()}</text>
                        {lanes}
                    </g>
                }
            })
            .collect_view();
        Some(view! {
            <svg node_ref=ctx.svg class="gx-plot__svg" xmlns="http://www.w3.org/2000/svg" width=f(w) height=f(h) viewBox=format!("0 0 {w:.0} {h:.0}")>
                {ticks}
                {rows}
                <line class="gx-base" x1=f(left) x2=f(right) y1=f(bottom) y2=f(bottom)/>
                <text class="gx-axis-title" x=f((left + right) / 2.0) y=f(bottom + 33.0) text-anchor="middle">"ms from the start of the iteration"</text>
            </svg>
        })
    };
    let tip = move || {
        let spot = hover.get()?;
        if drag.get().is_some() {
            return None;
        }
        let m = model.read();
        let r = m.rows.get(spot.row)?;
        let (tops, _) = row_tops(&m);
        let w = width.get();
        let refresh = m.refresh_ms;
        let mut lines: Vec<(String, String)> = Vec::new();
        if let Some(l) = spot.lane.and_then(|i| r.lanes.get(i)) {
            match spot.gap.and_then(|g| l.gaps.get(g)) {
                Some(g) => {
                    lines.push(("presented".into(), format!("{} ms", short(g.t_ms))));
                    lines.push((
                        "interval".into(),
                        format!(
                            "{} ms · {} refreshes",
                            short(g.interval_ms),
                            short(g.interval_ms / refresh)
                        ),
                    ));
                    if g.missed > 0 {
                        lines.push((
                            "missed".into(),
                            format!("{} · {}", g.missed, GAP_CLASSES[gap_class(g.missed)].name),
                        ));
                    }
                    if g.stalled {
                        lines.push((
                            "content".into(),
                            "stalled: the input moved, the content stayed".into(),
                        ));
                    }
                    if g.double_step {
                        lines.push(("content".into(), "moved two steps at once".into()));
                    }
                }
                None => {
                    let late = l.gaps.iter().filter(|g| g.missed > 0).count();
                    let worst = l.gaps.iter().map(|g| g.interval_ms).fold(0.0, f64::max);
                    lines.push(("iteration".into(), l.index.to_string()));
                    lines.push((
                        "frames".into(),
                        format!("{} presented · {late} late", l.frames),
                    ));
                    if worst > 0.0 {
                        lines.push(("longest listed".into(), format!("{} ms", short(worst))));
                    }
                    if let Some((a, b)) = l.presents {
                        lines.push(("presents".into(), format!("{} – {} ms", short(a), short(b))));
                    }
                    match l.base_missed {
                        Some(_) if l.frames < 2 => {
                            lines.push(("presents".into(), "too few to time a frame".into()));
                        }
                        Some(m) => {
                            lines.push(("typical frame".into(), format!("misses {m} refreshes")))
                        }
                        None => {}
                    }
                    if !l.complete {
                        lines.push((
                            "listed".into(),
                            format!("frames over {} ms only", short(l.threshold_ms)),
                        ));
                    }
                }
            }
        }
        let y = tops[spot.row] + row_height(r);
        Some(view! {
            <div class="gx-tip" style:left=format!("{:.0}px", (w * 0.45).min(w - 280.0).max(0.0)) style:top=format!("{y:.0}px")>
                <div class="gx-tip__head"><b style:color=r.ink.clone()>{r.label.clone()}</b></div>
                <div class="gx-tip__lines">
                    {lines.into_iter().map(|(k, v)| view! { <span>{k}</span><b>{v}</b> }).collect_view()}
                </div>
                {r.cell.is_some().then(|| view! { <div class="gx-tip__foot">"click: open the cell"</div> })}
            </div>
        })
    };
    let brush = move || {
        let d = drag.get()?;
        let h = model.with(|m| row_tops(m).1);
        Some(view! {
            <div class="gx-brush" style:left=format!("{:.1}px", d.x0.min(d.x1)) style:top="0px" style:width=format!("{:.1}px", (d.x1 - d.x0).abs()) style:height=format!("{h:.0}px")></div>
        })
    };
    view! {
        <div
            class="gx-plot gx-plot--lanes"
            node_ref=wrap
            style:height=move || format!("{:.0}px", model.with(|m| row_tops(m).1) + AXIS)
            on:pointerdown=move |ev: web_sys::PointerEvent| {
                if ev.button() != 0 { return; }
                if let Some(el) = wrap.get_untracked() {
                    let (x, y) = local(&el, &ev);
                    let _ = el.set_pointer_capture(ev.pointer_id());
                    drag.set(Some(Drag { x0: x, y0: y, x1: x, y1: y, pan: false }));
                }
            }
            on:pointermove=move |ev: web_sys::PointerEvent| {
                let Some(el) = wrap.get_untracked() else { return };
                let (x, y) = local(&el, &ev);
                if let Some(d) = drag.get_untracked() {
                    drag.set(Some(Drag { x1: x, y1: y, ..d }));
                    return;
                }
                let spot = spot_at(x, y);
                if hover.get_untracked() != spot {
                    hover.set(spot);
                }
                let group = spot.and_then(|s| model.with_untracked(|m| m.rows.get(s.row).map(|r| r.group.clone())));
                if ctx.highlight.get_untracked() != group {
                    ctx.highlight.set(group);
                }
            }
            on:pointerup=move |_| {
                let Some(d) = drag.get_untracked() else { return };
                drag.set(None);
                if (d.x1 - d.x0).abs() > 5.0 {
                    zoom.set(Some(scale_now().between(d.x0.min(d.x1), d.x0.max(d.x1))));
                } else if let Some(cell) = hover.get_untracked().and_then(|s| model.with_untracked(|m| m.rows.get(s.row).and_then(|r| r.cell))) {
                    explorer.query.set("cell", Some(cell.to_string()));
                }
            }
            on:pointerleave=move |_| { hover.set(None); ctx.highlight.set(None); }
            on:dblclick=move |_| zoom.set(None)
        >
            {svg}
            {brush}
            {tip}
        </div>
    }
}
