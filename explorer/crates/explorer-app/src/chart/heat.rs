//! A heatmap: rows by columns of cells, each colored on a sequential ramp (magnitude) or a
//! diverging one (a ratio around 1), with the value as text in a contrasting ink.

use explorer_core::api::CellId;
use leptos::html;
use leptos::prelude::*;

use super::card::use_card;
use super::size::use_width;
use crate::state::use_explorer;

/// One cell of a heatmap.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatCell {
    pub text: String,
    /// The color position: 0 to 1 on a sequential ramp, -1 to 1 on a diverging one.
    pub t: Option<f64>,
    pub cell: Option<CellId>,
    pub invalid: bool,
    pub tip: Vec<(String, String)>,
}

/// A heatmap.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatModel {
    /// Row labels with their ink.
    pub rows: Vec<(String, String)>,
    pub cols: Vec<String>,
    /// `cells[row][col]`.
    pub cells: Vec<Vec<Option<HeatCell>>>,
    pub diverging: bool,
    /// The meaning of the ends of the color ramp, such as ("cheaper", "costlier").
    pub ends: (String, String),
}

const CELL_H: f64 = 27.0;
const HEAD: f64 = 40.0;

/// Returns the fill of a color position.
pub fn fill(t: f64, diverging: bool) -> String {
    if diverging {
        let t = t.clamp(-1.0, 1.0);
        let pole = if t < 0.0 {
            "var(--div-lo)"
        } else {
            "var(--div-hi)"
        };
        format!(
            "color-mix(in oklab, {pole} {:.0}%, var(--div-mid))",
            t.abs() * 100.0
        )
    } else {
        let t = t.clamp(0.0, 1.0) * 4.0;
        let i = (t.floor() as usize).min(3);
        let local = t - i as f64;
        format!(
            "color-mix(in oklab, var(--seq-{}) {:.0}%, var(--seq-{i}))",
            i + 1,
            local * 100.0
        )
    }
}

fn ink(t: f64, diverging: bool) -> &'static str {
    let strong = if diverging { t.abs() > 0.62 } else { t > 0.55 };
    if strong {
        "var(--seq-text-dark)"
    } else {
        "var(--seq-text-light)"
    }
}

fn f(v: f64) -> String {
    format!("{v:.1}")
}

/// A heatmap that fills the width of its card.
#[component]
pub fn Heatmap(#[prop(into)] model: Signal<HeatModel>) -> impl IntoView {
    let ctx = use_card();
    let explorer = use_explorer();
    // The model builds once per change; every reader below shares it.
    let model = Memo::new(move |_| model.get());
    let wrap = NodeRef::<html::Div>::new();
    let width = use_width(wrap, ctx.width_hint);
    let hover = RwSignal::new(None::<(usize, usize)>);
    Effect::new(move |_| {
        let m = model.read();
        let mut csv = format!("row,{}\n", m.cols.join(","));
        for (r, (label, _)) in m.rows.iter().enumerate() {
            let cells: Vec<String> = m.cells[r]
                .iter()
                .map(|c| c.as_ref().map(|c| c.text.clone()).unwrap_or_default())
                .collect();
            csv.push_str(&format!("{label},{}\n", cells.join(",")));
        }
        ctx.csv.set(csv);
    });
    let layout = Memo::new(move |_| {
        let w = width.get();
        let m = model.read();
        let label_w = m.rows.iter().map(|(l, _)| l.len()).max().unwrap_or(4) as f64 * 7.2 + 16.0;
        let n = m.cols.len().max(1) as f64;
        let col_w = ((w - label_w) / n).clamp(34.0, 220.0);
        (w, label_w, col_w)
    });
    let svg = move || {
        let (w, label_w, col_w) = layout.get();
        if w < 40.0 {
            return None;
        }
        let m = model.read();
        let h = HEAD + m.rows.len() as f64 * CELL_H + 22.0;
        let hl = ctx.highlight;
        let heads = m
            .cols
            .iter()
            .enumerate()
            .map(|(c, label)| {
                let cx = label_w + (c as f64 + 0.5) * col_w;
                let lines: Vec<String> = label.split('\n').map(str::to_owned).collect();
                let n = lines.len() as f64;
                lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, line)| view! {
                        <text class="gx-heat__col" x=f(cx) y=f(HEAD - 7.0 - (n - 1.0 - i as f64) * 14.0) text-anchor="middle">{line}</text>
                    })
                    .collect_view()
            })
            .collect_view();
        let body = m
            .rows
            .iter()
            .enumerate()
            .map(|(r, (label, ink_color))| {
                let y = HEAD + r as f64 * CELL_H;
                let row = label.clone();
                let dim = move || hl.with(|h| h.as_ref().is_some_and(|h| *h != row)).then_some("");
                let cells = m.cells[r]
                    .iter()
                    .enumerate()
                    .map(|(c, cell)| {
                        let x = label_w + c as f64 * col_w;
                        match cell {
                            Some(cell) => {
                                let t = cell.t.unwrap_or(0.0);
                                let bg = cell.t.map_or("var(--band)".to_owned(), |t| fill(t, m.diverging));
                                let fg = if cell.t.is_some() { ink(t, m.diverging) } else { "var(--muted)" };
                                view! {
                                    <rect class="gx-heat__cell" data-invalid=cell.invalid.then_some("") x=f(x + 1.0) y=f(y + 1.0) width=f(col_w - 2.0) height=f(CELL_H - 2.0) rx="3" style:fill=bg/>
                                    <text class="gx-heat__text" x=f(x + col_w / 2.0) y=f(y + CELL_H / 2.0 + 0.5) text-anchor="middle" dominant-baseline="middle" style:fill=fg>{cell.text.clone()}</text>
                                }
                                .into_any()
                            }
                            None => view! {
                                <rect class="gx-heat__empty" x=f(x + 1.0) y=f(y + 1.0) width=f(col_w - 2.0) height=f(CELL_H - 2.0) rx="3"/>
                            }
                            .into_any(),
                        }
                    })
                    .collect_view();
                view! {
                    <g class="gx-heat__row" data-dim=dim>
                        <text class="gx-rowlabel" x=f(label_w - 8.0) y=f(y + CELL_H / 2.0) text-anchor="end" dominant-baseline="middle" style:fill=ink_color.clone()>{label.clone()}</text>
                        {cells}
                    </g>
                }
            })
            .collect_view();
        let ramp_y = h - 12.0;
        let ramp = (0..5)
            .map(|i| {
                let t = if m.diverging { -1.0 + i as f64 * 0.5 } else { i as f64 / 4.0 };
                view! { <rect x=f(label_w + i as f64 * 18.0) y=f(ramp_y - 6.0) width="17" height="8" rx="2" style:fill=fill(t, m.diverging)/> }
            })
            .collect_view();
        let (lo, hi) = m.ends.clone();
        Some(view! {
            <svg node_ref=ctx.svg class="gx-plot__svg" xmlns="http://www.w3.org/2000/svg" width=f(w) height=f(h) viewBox=format!("0 0 {w:.0} {h:.0}")>
                {heads}
                {body}
                <text class="gx-tick" x=f(label_w - 6.0) y=f(ramp_y) text-anchor="end">{lo}</text>
                {ramp}
                <text class="gx-tick" x=f(label_w + 96.0) y=f(ramp_y)>{hi}</text>
            </svg>
        })
    };
    let tip = move || {
        let (r, c) = hover.get()?;
        let m = model.read();
        let cell = m.cells.get(r)?.get(c)?.clone()?;
        let (_, label_w, col_w) = layout.get();
        let x = label_w + (c as f64 + 1.0) * col_w;
        let y = HEAD + (r as f64 + 1.0) * CELL_H;
        let (row, row_ink) = m.rows[r].clone();
        Some(view! {
            <div class="gx-tip" data-flip=(c + 2 > m.cols.len() && c > 0).then_some("") style:left=format!("{:.0}px", if c + 2 > m.cols.len() && c > 0 { x - col_w } else { x }) style:top=format!("{y:.0}px")>
                <div class="gx-tip__head"><b style:color=row_ink>{row}</b><span class="gx-muted">{format!(" · {}", m.cols[c].replace('\n', " "))}</span></div>
                <div class="gx-tip__lines">
                    {cell.tip.iter().map(|(k, v)| view! { <span>{k.clone()}</span><b>{v.clone()}</b> }).collect_view()}
                </div>
                {cell.invalid.then(|| view! { <div class="gx-tip__warn">"the report fails a validity check"</div> })}
                {cell.cell.is_some().then(|| view! { <div class="gx-tip__foot">"click: open the cell"</div> })}
            </div>
        })
    };
    let hit = move |ev: &web_sys::MouseEvent| -> Option<(usize, usize)> {
        let el = wrap.get_untracked()?;
        let (x, y) = super::interact::local(&el, ev);
        let (_, label_w, col_w) = layout.get_untracked();
        if x < label_w || y < HEAD {
            return None;
        }
        let c = ((x - label_w) / col_w).floor() as usize;
        let r = ((y - HEAD) / CELL_H).floor() as usize;
        model.with_untracked(|m| (r < m.rows.len() && c < m.cols.len()).then_some((r, c)))
    };
    view! {
        <div
            class="gx-plot gx-plot--heat"
            node_ref=wrap
            style:height=move || format!("{:.0}px", model.with(|m| HEAD + m.rows.len() as f64 * CELL_H + 22.0))
            on:pointermove=move |ev: web_sys::PointerEvent| {
                let h = hit(&ev);
                hover.set(h);
                let group = h.map(|(r, _)| model.with_untracked(|m| m.rows[r].0.clone()));
                if ctx.highlight.get_untracked() != group {
                    ctx.highlight.set(group);
                }
            }
            on:pointerleave=move |_| { hover.set(None); ctx.highlight.set(None); }
            on:click=move |ev: web_sys::MouseEvent| {
                if let Some((r, c)) = hit(&ev)
                    && let Some(cell) = model.with_untracked(|m| m.cells[r][c].as_ref().and_then(|c| c.cell))
                {
                    explorer.query.set("cell", Some(cell.to_string()));
                }
            }
        >
            {svg}
            {tip}
        </div>
    }
}
