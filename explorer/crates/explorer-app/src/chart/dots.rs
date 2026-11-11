//! A dot plot: one row per framework and variant (or per metric, in the compare view), a dot for
//! the value with a band and whiskers, alternating bands per group, value labels, and an x axis
//! that zooms. Focus mode pins outlying rows to the right edge.

use explorer_core::api::CellId;
use explorer_core::scale::{Scale, ScaleKind, padded};
use explorer_core::stats::focus_limit;
use explorer_core::units::{short, thousands};
use leptos::html;
use leptos::prelude::*;

use super::card::use_card;
use super::interact::{Drag, local, on_zoom_wheel};
use super::model::{Axis, Rule};
use super::size::use_width;
use crate::palette::Marker;
use crate::state::use_explorer;

/// One row of a dot plot.
#[derive(Debug, Clone, PartialEq)]
pub struct DotRow {
    pub key: String,
    pub label: String,
    /// The framework, for shading, highlight and the legend.
    pub group: String,
    pub color: String,
    pub soft: String,
    pub ink: String,
    pub marker: Marker,
    pub hollow: bool,
    pub value: Option<f64>,
    pub band: Option<(f64, f64)>,
    pub whisker: Option<(f64, f64)>,
    /// The value label; empty prints the value.
    pub text: String,
    pub cell: Option<CellId>,
    pub invalid: bool,
    /// The value of the compare run.
    pub ghost: Option<f64>,
    /// The row is a section heading.
    pub header: bool,
    pub tip: Vec<(String, String)>,
}

impl DotRow {
    /// Returns a row in the colors of a framework.
    pub fn of_framework(key: impl Into<String>, framework: &str, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            group: framework.to_owned(),
            color: crate::palette::mark(framework),
            soft: crate::palette::soft(framework),
            ink: crate::palette::ink(framework),
            marker: Marker::Circle,
            hollow: false,
            value: None,
            band: None,
            whisker: None,
            text: String::new(),
            cell: None,
            invalid: false,
            ghost: None,
            header: false,
            tip: Vec::new(),
        }
    }

    /// Returns a section heading.
    pub fn header(label: impl Into<String>) -> Self {
        let mut row = Self::of_framework(String::new(), "", label);
        row.header = true;
        row.key = format!("h:{}", row.label);
        row
    }
}

/// A dot plot.
#[derive(Debug, Clone, PartialEq)]
pub struct DotModel {
    pub rows: Vec<DotRow>,
    pub x: Axis,
    pub rules: Vec<Rule>,
    /// Hide the row labels, such as in the second of several panels.
    pub hide_labels: bool,
    /// The axis holds ratios: ticks read as factors of at least 1, so 0.5 reads 2×.
    pub factors: bool,
}

const ROW: f64 = 22.0;
const TOP: f64 = 6.0;
const AXIS: f64 = 38.0;

struct Geo {
    width: f64,
    height: f64,
    left: f64,
    right: f64,
    x: Scale,
    focused: bool,
}

fn geo(m: &DotModel, width: f64, kind: ScaleKind, focus: bool, zoom: Option<(f64, f64)>) -> Geo {
    let log = kind == ScaleKind::Log;
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut peaks = Vec::new();
    for r in m.rows.iter().filter(|r| !r.header) {
        for v in [r.value, r.ghost]
            .into_iter()
            .flatten()
            .chain(r.band.into_iter().flat_map(|(a, b)| [a, b]))
            .chain(r.whisker.into_iter().flat_map(|(a, b)| [a, b]))
        {
            if v.is_finite() && (!log || v > 0.0) {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        if let Some(v) = r.value {
            peaks.push(v);
        }
    }
    for rule in &m.rules {
        lo = lo.min(rule.value);
        hi = hi.max(rule.value);
    }
    if lo > hi {
        (lo, hi) = (0.0, 1.0);
    }
    let mut focused = false;
    if focus
        && let Some(top) = focus_limit(&peaks.iter().map(|v| v.abs()).collect::<Vec<_>>())
        && top < hi
    {
        hi = top;
        focused = true;
    }
    let mut domain =
        m.x.domain
            .unwrap_or_else(|| padded(kind, lo, hi, m.x.zero && !log));
    if !log && m.x.zero && lo >= 0.0 {
        domain.0 = 0.0;
    }
    if let Some(z) = zoom {
        domain = z;
    }
    let label_w = if m.hide_labels {
        8.0
    } else {
        m.rows
            .iter()
            .filter(|r| !r.header)
            .map(|r| r.label.len())
            .max()
            .unwrap_or(4) as f64
            * 7.2
            + 14.0
    };
    let right = (width - 58.0).max(label_w + 60.0);
    let rows = m.rows.len() as f64;
    Geo {
        width,
        height: TOP + rows * ROW + AXIS,
        left: label_w,
        right,
        x: Scale::new(kind, domain, (label_w, right)),
        focused,
    }
}

/// Returns ticks of a ratio axis at clean factors mirrored around 1: 1, 2, 3, 5, 10 … and their
/// reciprocals, thinned to about `count`.
fn factor_ticks(x: &Scale, count: usize) -> Vec<explorer_core::scale::Tick> {
    const STEPS: [f64; 4] = [1.0, 2.0, 3.0, 5.0];
    let (lo, hi) = x.bounds();
    let mut factors: Vec<f64> = (0..5)
        .flat_map(|d| STEPS.iter().map(move |s| s * 10f64.powi(d)))
        .collect();
    factors.dedup();
    let mut values: Vec<f64> = factors
        .iter()
        .flat_map(|f| [*f, 1.0 / f])
        .filter(|v| *v >= lo && *v <= hi)
        .collect();
    values.sort_by(f64::total_cmp);
    values.dedup();
    // Thin to decades and their halves first, then decades only, while too many remain.
    for keep in [&[1.0, 2.0, 3.0, 5.0][..], &[1.0, 3.0][..], &[1.0][..]] {
        let kept: Vec<f64> = values
            .iter()
            .copied()
            .filter(|v| {
                let f = if *v < 1.0 { 1.0 / v } else { *v };
                let mantissa = f / 10f64.powf(f.log10().floor());
                keep.iter().any(|k| (mantissa - k).abs() < 1e-6)
            })
            .collect();
        if kept.len() <= count.max(3) || keep.len() == 1 {
            values = kept;
            break;
        }
    }
    values
        .into_iter()
        .map(|value| explorer_core::scale::Tick { value, major: true })
        .collect()
}

fn f(v: f64) -> String {
    format!("{v:.1}")
}

/// A dot plot that fills the width of its card.
#[component]
pub fn DotPlot(#[prop(into)] model: Signal<DotModel>) -> impl IntoView {
    let ctx = use_card();
    let explorer = use_explorer();
    // The model builds once per change; every reader below shares it.
    let model = Memo::new(move |_| model.get());
    let wrap = NodeRef::<html::Div>::new();
    let width = use_width(wrap, ctx.width_hint);
    let zoom = RwSignal::new(None::<(f64, f64)>);
    let hover = RwSignal::new(None::<usize>);
    let drag = RwSignal::new(None::<Drag>);
    Effect::new(move |_| {
        let m = model.read();
        ctx.has_scale.set(m.x.value);
        ctx.scale_default.set(m.x.kind);
        let mut csv = String::from("row,value,band_lo,band_hi,whisker_lo,whisker_hi,cell\n");
        for r in m.rows.iter().filter(|r| !r.header) {
            let o = |v: Option<f64>| v.map(|v| v.to_string()).unwrap_or_default();
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                r.label,
                o(r.value),
                o(r.band.map(|b| b.0)),
                o(r.band.map(|b| b.1)),
                o(r.whisker.map(|b| b.0)),
                o(r.whisker.map(|b| b.1)),
                r.cell.map(|c| c.to_string()).unwrap_or_default()
            ));
        }
        ctx.csv.set(csv);
    });
    let kind = Memo::new(move |_| {
        let m = model.read();
        if m.x.value {
            ctx.scale_kind(m.x.kind)
        } else {
            m.x.kind
        }
    });
    Effect::new(move |prev: Option<ScaleKind>| {
        let k = kind.get();
        if prev.is_some_and(|p| p != k) {
            zoom.set(None);
        }
        k
    });
    Effect::new(move |_| ctx.zoomed.set(zoom.get().is_some()));
    Effect::new(move |prev: Option<u32>| {
        let n = ctx.reset.get();
        if prev.is_some() {
            zoom.set(None);
        }
        n
    });
    let g = Memo::new(move |_| {
        let w = width.get();
        (w >= 40.0).then(|| {
            let (k, f, z) = (kind.get(), ctx.focus_on(), zoom.get());
            let m = model.read();
            let g = geo(&m, w, k, f, z);
            (g.width, g.height, g.left, g.right, g.x.domain, g.focused)
        })
    });
    let scale = move || {
        g.get()
            .map(|(_, _, l, r, d, _)| Scale::new(kind.get(), d, (l, r)))
    };
    let zoom_by = move |factor: f64, at: Option<f64>| {
        if let (Some(s), Some((_, _, l, r, _, _))) = (scale(), g.get_untracked()) {
            zoom.set(Some(s.zoomed(at.unwrap_or((l + r) / 2.0), factor)));
        }
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
            zoom_by(if ev.delta_y() < 0.0 { 0.85 } else { 1.0 / 0.85 }, Some(x));
        }
    });
    let row_at = move |y: f64| -> Option<usize> {
        let i = ((y - TOP) / ROW).floor();
        (i >= 0.0)
            .then_some(i as usize)
            .filter(|i| model.with_untracked(|m| m.rows.get(*i).is_some_and(|r| !r.header)))
    };
    let svg = move || {
        let (w, h, left, right, domain, focused) = g.get()?;
        let x = Scale::new(kind.get(), domain, (left, right));
        let m = model.read();
        let hl = ctx.highlight;
        let (_, top_value) = x.bounds();
        let bottom = TOP + m.rows.len() as f64 * ROW;
        let mut shade = false;
        let mut prev_group = String::new();
        let bands = m
            .rows
            .iter()
            .enumerate()
            .map(|(i, r)| {
                if r.group != prev_group || r.header {
                    shade = !shade && !r.header;
                    prev_group = r.group.clone();
                }
                let y = TOP + i as f64 * ROW;
                (shade && !r.header).then(|| view! { <rect class="gx-rowband" x="0" y=f(y) width=f(right + 6.0) height=f(ROW)/> })
            })
            .collect_view();
        let factors = m.factors;
        let tick = move |v: f64| {
            if factors {
                let f = if v > 0.0 && v < 1.0 { 1.0 / v } else { v };
                format!("{}×", thousands(f))
            } else {
                thousands(v)
            }
        };
        let count = ((right - left) / 70.0).round().max(2.0) as usize;
        let ticks = if factors {
            factor_ticks(&x, count)
        } else {
            x.ticks(count)
        };
        let ticks = ticks
            .into_iter()
            .map(|t| {
                let px = f(x.map(t.value));
                view! {
                    <line class="gx-gridline" data-minor=(!t.major).then_some("") x1=px.clone() x2=px.clone() y1=f(TOP) y2=f(bottom)/>
                    {t.major.then(|| view! { <text class="gx-tick" x=px y=f(bottom + 16.0) text-anchor="middle">{tick(t.value)}</text> })}
                }
            })
            .collect_view();
        let rules = m
            .rules
            .iter()
            .filter(|r| x.contains(r.value))
            .map(|r| {
                let px = f(x.map(r.value));
                view! { <g class="gx-rule gx-rule--solid"><line x1=px.clone() x2=px y1=f(TOP - 2.0) y2=f(bottom)/></g> }
            })
            .collect_view();
        let rows = m
            .rows
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let cy = TOP + i as f64 * ROW + ROW / 2.0;
                if r.header {
                    return view! { <text class="gx-rowhead" x="2" y=f(cy + 1.0) dominant-baseline="middle">{r.label.clone()}</text> }.into_any();
                }
                let group = r.group.clone();
                let dim = move || hl.with(|h| h.as_ref().is_some_and(|h| *h != group)).then_some("");
                let label = (!m.hide_labels).then(|| view! {
                    <text class="gx-rowlabel" x=f(left - 8.0) y=f(cy) text-anchor="end" dominant-baseline="middle" style:fill=r.ink.clone()>{r.label.clone()}</text>
                });
                let Some(v) = r.value else {
                    return view! { {label}<text class="gx-nodata" x=f(left + 4.0) y=f(cy) dominant-baseline="middle">"no data"</text> }.into_any();
                };
                let clamp = |v: f64| x.map(v.min(top_value)).clamp(left, right);
                let whisker = r.whisker.map(|(a, b)| view! {
                    <line class="gx-whisker" x1=f(clamp(a)) x2=f(clamp(b)) y1=f(cy) y2=f(cy) style:stroke=r.soft.clone()/>
                });
                let band = r.band.map(|(a, b)| view! {
                    <line class="gx-iqr" x1=f(clamp(a)) x2=f(clamp(b)) y1=f(cy) y2=f(cy) style:stroke=r.soft.clone()/>
                });
                let ghost = r.ghost.filter(|g| g.is_finite()).map(|gv| view! {
                    <path class="gx-marker gx-marker--ghost" d=r.marker.path(3.6) transform=format!("translate({:.1} {cy:.1})", clamp(gv)) style:stroke=r.color.clone()/>
                });
                let out = v > top_value && focused;
                let px = clamp(v);
                let hollow = r.hollow || r.invalid;
                let mark = if out {
                    view! { <path class="gx-pinmark" d=format!("M{:.1},{:.1}l6,4.5l-6,4.5z", right + 1.0, cy - 4.5) style:fill=r.color.clone()/> }.into_any()
                } else {
                    view! {
                        <path
                            class="gx-marker"
                            data-invalid=r.invalid.then_some("")
                            d=r.marker.path(4.4)
                            transform=format!("translate({px:.1} {cy:.1})")
                            style:fill=if hollow { "var(--surface)".to_owned() } else { r.color.clone() }
                            style:stroke=if hollow { r.color.clone() } else { "var(--surface)".to_owned() }
                        />
                    }.into_any()
                };
                let text = if r.text.is_empty() { short(v) } else { r.text.clone() };
                let end = r.whisker.map_or(px, |(_, b)| clamp(b)).max(r.band.map_or(px, |(_, b)| clamp(b))).max(px);
                let tx = if out { right + 10.0 } else { end + 6.0 };
                view! {
                    <g class="gx-row" data-dim=dim>
                        {label}
                        {whisker}
                        {band}
                        {ghost}
                        {mark}
                        <text class="gx-rowvalue" x=f(tx) y=f(cy) dominant-baseline="middle" style:fill=r.ink.clone()>{text}</text>
                    </g>
                }
                .into_any()
            })
            .collect_view();
        let title = m.x.label.clone();
        Some(view! {
            <svg node_ref=ctx.svg class="gx-plot__svg" xmlns="http://www.w3.org/2000/svg" width=f(w) height=f(h) viewBox=format!("0 0 {w:.0} {h:.0}")>
                {bands}
                {ticks}
                {rules}
                <line class="gx-base" x1=f(left) x2=f(right) y1=f(bottom) y2=f(bottom)/>
                {rows}
                <text class="gx-axis-title" x=f((left + right) / 2.0) y=f(bottom + 33.0) text-anchor="middle">{title}</text>
            </svg>
        })
    };
    let tip = move || {
        let i = hover.get()?;
        if drag.get().is_some() {
            return None;
        }
        let m = model.read();
        let r = m.rows.get(i)?.clone();
        let (w, ..) = g.get()?;
        let unit = m.x.unit.clone();
        let fmt = move |v: f64| {
            if unit.is_empty() {
                short(v)
            } else {
                format!("{} {unit}", short(v))
            }
        };
        let y = TOP + i as f64 * ROW + ROW;
        Some(view! {
            <div class="gx-tip" style:left=format!("{:.0}px", (w * 0.45).min(w - 260.0).max(0.0)) style:top=format!("{y:.0}px")>
                <div class="gx-tip__head"><span class="gx-tip__dot" style:background=r.color.clone()></span><b style:color=r.ink.clone()>{r.label.clone()}</b></div>
                <div class="gx-tip__lines">
                    {(!m.factors).then(|| view! {
                        {r.value.map(|v| view! { <span>{m.x.label.clone()}</span><b>{fmt(v)}</b> })}
                        {r.band.map(|(a, b)| view! { <span>"band"</span><b>{format!("{} – {}", short(a), fmt(b))}</b> })}
                        {r.whisker.map(|(a, b)| view! { <span>"whiskers"</span><b>{format!("{} – {}", short(a), fmt(b))}</b> })}
                    })}
                    {r.ghost.map(|v| view! { <span>"compare run"</span><b>{fmt(v)}</b> })}
                    {r.tip.iter().map(|(k, v)| view! { <span>{k.clone()}</span><b>{v.clone()}</b> }).collect_view()}
                </div>
                {r.invalid.then(|| view! { <div class="gx-tip__warn">"the report fails a validity check"</div> })}
                {r.cell.is_some().then(|| view! { <div class="gx-tip__foot">"click: open the cell"</div> })}
            </div>
        })
    };
    let brush = move || {
        let d = drag.get()?;
        let (_, h, ..) = g.get()?;
        Some(view! {
            <div class="gx-brush" style:left=format!("{:.1}px", d.x0.min(d.x1)) style:top="0px" style:width=format!("{:.1}px", (d.x1 - d.x0).abs()) style:height=format!("{:.0}px", h - AXIS)></div>
        })
    };
    view! {
        <div
            class="gx-plot gx-plot--dots"
            node_ref=wrap
            style:height=move || format!("{:.0}px", model.with(|m| TOP + m.rows.len() as f64 * ROW + AXIS))
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
                let row = row_at(y);
                hover.set(row);
                let group = row.and_then(|i| model.with_untracked(|m| m.rows.get(i).map(|r| r.group.clone())));
                if ctx.highlight.get_untracked() != group {
                    ctx.highlight.set(group);
                }
            }
            on:pointerup=move |_| {
                let Some(d) = drag.get_untracked() else { return };
                drag.set(None);
                if (d.x1 - d.x0).abs() > 5.0 {
                    if let Some(s) = scale() {
                        zoom.set(Some(s.between(d.x0.min(d.x1), d.x0.max(d.x1))));
                    }
                } else if let Some(cell) = hover.get_untracked().and_then(|i| model.with_untracked(|m| m.rows.get(i).and_then(|r| r.cell))) {
                    explorer.query.set("cell", Some(cell.to_string()));
                }
            }
            on:pointerleave=move |_| { hover.set(None); ctx.highlight.set(None); }
            on:dblclick=move |_| zoom.set(None)
        >
            {svg}
            {move || hover.get().map(|i| view! { <div class="gx-rowhover" style:top=format!("{:.0}px", TOP + i as f64 * ROW)></div> })}
            {brush}
            {tip}
        </div>
    }
}
