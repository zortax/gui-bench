//! The x-y plot component. It lays out its scene from the model, the card preferences, its
//! width and its viewport, and handles hover, click, box zoom, pan and ctrl+wheel zoom.

use std::sync::atomic::{AtomicUsize, Ordering};

use explorer_core::scale::ScaleKind;
use leptos::html;
use leptos::prelude::*;

use super::super::card::use_card;
use super::super::interact::{Drag, Viewport, local, on_zoom_wheel};
use super::super::model::XyModel;
use super::super::size::use_width;
use super::layers;
use super::scene::{Hit, Scene, SceneOpts, layout};
use super::tooltip;
use crate::state::use_explorer;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// Returns the pixel room that direct end labels need.
fn label_room(model: &XyModel) -> f64 {
    model
        .series
        .iter()
        .filter(|s| s.end_label && !s.ghost)
        .map(|s| s.label.len() as f64 * 7.8 + 70.0)
        .fold(0.0, f64::max)
}

/// Returns the shown data of a model as CSV.
fn csv(model: &XyModel) -> String {
    let mut out = String::from("series,x,y,band_lo,band_hi,whisker_lo,whisker_hi,cell\n");
    let opt = |v: Option<(f64, f64)>| {
        v.map_or((String::new(), String::new()), |(a, b)| {
            (a.to_string(), b.to_string())
        })
    };
    for s in &model.series {
        for p in &s.points {
            let (bl, bh) = opt(p.band);
            let (wl, wh) = opt(p.whisker);
            let cell = p.cell.map(|c| c.to_string()).unwrap_or_default();
            out.push_str(&format!(
                "{},{},{},{bl},{bh},{wl},{wh},{cell}\n",
                s.label, p.x, p.y
            ));
        }
    }
    for b in &model.bars {
        out.push_str(&format!(
            "{},{}-{},{}-{},,,,\n",
            b.group, b.x0, b.x1, b.y0, b.y1
        ));
    }
    out
}

fn zoom_range(scale: &explorer_core::scale::Scale, at: f64, factor: f64) -> (f64, f64) {
    scale.zoomed(at, factor)
}

/// An x-y plot that fills the width of its card.
#[component]
pub fn XyPlot(#[prop(into)] model: Signal<XyModel>) -> impl IntoView {
    let ctx = use_card();
    let explorer = use_explorer();
    // The model builds once per change; every reader below shares it.
    let model = Memo::new(move |_| model.get());
    let wrap = NodeRef::<html::Div>::new();
    let width = use_width(wrap, ctx.width_hint);
    let viewport = RwSignal::new(Viewport::default());
    let hover = RwSignal::new(None::<Hit>);
    let drag = RwSignal::new(None::<Drag>);
    let clip = format!("gx-clip-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));

    Effect::new(move |_| {
        model.with(|m| {
            ctx.has_scale.set(m.y.value || m.x.value);
            ctx.scale_default
                .set(if m.x.value { m.x.kind } else { m.y.kind });
            ctx.csv.set(csv(m));
        });
    });
    let y_kind = Memo::new(move |_| {
        let (value, kind) = model.with(|m| (m.y.value, m.y.kind));
        if value { ctx.scale_kind(kind) } else { kind }
    });
    let x_kind = Memo::new(move |_| {
        let (value, kind) = model.with(|m| (m.x.value, m.x.kind));
        if value { ctx.scale_kind(kind) } else { kind }
    });
    Effect::new(move |previous: Option<ScaleKind>| {
        let kind = x_kind.get();
        if previous.is_some_and(|p| p != kind) {
            viewport.update(|v| v.x = None);
        }
        kind
    });
    Effect::new(move |previous: Option<ScaleKind>| {
        let kind = y_kind.get();
        if previous.is_some_and(|p| p != kind) {
            viewport.update(|v| v.y = None);
        }
        kind
    });
    let focus = Memo::new(move |_| ctx.focus_on());
    let scene = Memo::new(move |_| {
        let w = width.get();
        if w < 40.0 {
            return None;
        }
        let (xk, yk, f, v) = (x_kind.get(), y_kind.get(), focus.get(), viewport.get());
        let m = model.read();
        Some(layout(
            &m,
            SceneOpts {
                width: w,
                x_kind: xk,
                y_kind: yk,
                focus: f,
                view: v,
                label_room: label_room(&m),
            },
        ))
    });
    Effect::new(move |_| ctx.zoomed.set(viewport.get().is_zoomed()));
    Effect::new(move |previous: Option<u32>| {
        let n = ctx.reset.get();
        if previous.is_some() {
            viewport.set(Viewport::default());
        }
        n
    });
    let zoom_center = move |factor: f64| {
        if let Some(sc) = scene.get_untracked() {
            let at = (sc.left + sc.right) / 2.0;
            viewport.update(|v| v.x = Some(zoom_range(&sc.x, at, factor)));
        }
    };
    Effect::new(move |previous: Option<u32>| {
        let n = ctx.zoom_in.get();
        if previous.is_some() {
            zoom_center(0.7);
        }
        n
    });
    Effect::new(move |previous: Option<u32>| {
        let n = ctx.zoom_out.get();
        if previous.is_some() {
            zoom_center(1.0 / 0.7);
        }
        n
    });

    on_zoom_wheel(wrap, move |ev: web_sys::WheelEvent| {
        let (Some(sc), Some(el)) = (scene.get_untracked(), wrap.get_untracked()) else {
            return;
        };
        let (px, py) = local(&el, &ev);
        let factor = if ev.delta_y() < 0.0 { 0.85 } else { 1.0 / 0.85 };
        if ev.shift_key() {
            viewport.update(|v| v.y = Some(zoom_range(&sc.y, py, factor)));
        } else {
            viewport.update(|v| v.x = Some(zoom_range(&sc.x, px, factor)));
        }
    });

    let on_down = move |ev: web_sys::PointerEvent| {
        if ev.button() != 0 {
            return;
        }
        let Some(el) = wrap.get_untracked() else {
            return;
        };
        let (x, y) = local(&el, &ev);
        if scene.get_untracked().is_some_and(|sc| sc.inside(x, y)) {
            let _ = el.set_pointer_capture(ev.pointer_id());
            drag.set(Some(Drag {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
                pan: ev.shift_key(),
            }));
        }
    };
    let on_move = move |ev: web_sys::PointerEvent| {
        let (Some(el), Some(sc)) = (wrap.get_untracked(), scene.get_untracked()) else {
            return;
        };
        let (x, y) = local(&el, &ev);
        if let Some(d) = drag.get_untracked() {
            if d.pan {
                let (dx, dy) = (x - d.x1, y - d.y1);
                viewport.update(|v| {
                    v.x = Some(sc.x.panned(dx));
                    if dy.abs() > 0.0 {
                        v.y = Some(sc.y.panned(dy));
                    }
                });
            }
            drag.set(Some(Drag { x1: x, y1: y, ..d }));
            return;
        }
        let (hit, group, crosshair) = model.with_untracked(|m| {
            let hit = if m.crosshair && sc.inside(x, y) {
                sc.nearest_x(x)
            } else {
                sc.nearest(x, y, 22.0)
            };
            let group = hit.and_then(|h| {
                if h.bar {
                    m.bars.get(h.index).map(|b| b.group.clone())
                } else {
                    m.series.get(h.series).map(|s| s.group.clone())
                }
            });
            (hit, group, m.crosshair)
        });
        if hover.get_untracked() != hit {
            hover.set(hit);
        }
        if !crosshair && ctx.highlight.get_untracked() != group {
            ctx.highlight.set(group);
        }
    };
    let on_up = move |ev: web_sys::PointerEvent| {
        let Some(d) = drag.get_untracked() else {
            return;
        };
        drag.set(None);
        let moved = (d.x1 - d.x0).abs().max((d.y1 - d.y0).abs()) > 4.0;
        let Some(sc) = scene.get_untracked() else {
            return;
        };
        if !moved {
            let cell = model.with_untracked(|m| {
                hover.get_untracked().and_then(|h| {
                    if h.bar {
                        m.bars.get(h.index).and_then(|b| b.cell)
                    } else {
                        m.series.get(h.series)?.points.get(h.index)?.cell
                    }
                })
            });
            if let Some(cell) = cell {
                explorer.query.set("cell", Some(cell.to_string()));
            }
            return;
        }
        if d.pan {
            return;
        }
        let _ = ev;
        let wide = (d.x1 - d.x0).abs() > 6.0;
        let tall = (d.y1 - d.y0).abs() > 6.0;
        viewport.update(|v| {
            if wide {
                v.x = Some(sc.x.between(d.x0.min(d.x1), d.x0.max(d.x1)));
            }
            if tall && (d.y1 - d.y0).abs() > 0.25 * (d.x1 - d.x0).abs() {
                v.y = Some(sc.y.between(d.y0.max(d.y1), d.y0.min(d.y1)));
            }
        });
    };
    let on_leave = move |_| {
        hover.set(None);
        ctx.highlight.set(None);
    };

    let svg = move || {
        scene.get().map(|sc: Scene| {
            let m = model.read();
            let hl = ctx.highlight;
            let clip = clip.clone();
            let clip_url = format!("url(#{clip})");
            view! {
                <svg
                    node_ref=ctx.svg
                    class="gx-plot__svg"
                    xmlns="http://www.w3.org/2000/svg"
                    width=format!("{:.0}", sc.width)
                    height=format!("{:.0}", sc.height)
                    viewBox=format!("0 0 {:.0} {:.0}", sc.width, sc.height)
                >
                    <defs>
                        <clipPath id=clip>
                            <rect
                                x=format!("{:.1}", sc.left - 6.0)
                                y=format!("{:.1}", sc.top - 6.0)
                                width=format!("{:.1}", sc.right - sc.left + 12.0)
                                height=format!("{:.1}", sc.bottom - sc.top + 6.0)
                            />
                        </clipPath>
                    </defs>
                    {layers::axes(&sc, &m)}
                    <g clip-path=clip_url>
                        {layers::bars(&sc, &m, hl)}
                        {layers::rules(&sc, &m)}
                        {layers::series(&sc, &m, hl)}
                    </g>
                    {layers::pins(&sc, &m)}
                    {layers::end_labels(&sc, &m, &m.y.unit)}
                </svg>
            }
        })
    };
    let ring = move || {
        let (h, sc) = (hover.get()?, scene.get()?);
        if h.bar || drag.get().is_some() {
            return None;
        }
        let line = model.with(|m| m.crosshair).then(|| view! {
            <div class="gx-cross" style:left=format!("{:.1}px", h.px) style:top=format!("{:.1}px", sc.top) style:height=format!("{:.1}px", sc.bottom - sc.top)></div>
        });
        Some(view! {
            {line}
            <div class="gx-ring" style:left=format!("{:.1}px", h.px) style:top=format!("{:.1}px", h.py)></div>
        })
    };
    let tip = move || {
        let (h, sc) = (hover.get()?, scene.get()?);
        if drag.get().is_some() {
            return None;
        }
        let m = model.read();
        let flip = h.px > sc.width - 240.0;
        Some(view! {
            <div
                class="gx-tip"
                data-flip=flip.then_some("")
                style:left=format!("{:.1}px", if flip { h.px - 14.0 } else { h.px + 14.0 })
                style:top=format!("{:.1}px", (h.py - 12.0).max(0.0))
            >
                {tooltip::body(&m, &sc, h)}
            </div>
        })
    };
    let brush = move || {
        let d = drag.get()?;
        if d.pan {
            return None;
        }
        Some(view! {
            <div
                class="gx-brush"
                style:left=format!("{:.1}px", d.x0.min(d.x1))
                style:top=format!("{:.1}px", d.y0.min(d.y1))
                style:width=format!("{:.1}px", (d.x1 - d.x0).abs())
                style:height=format!("{:.1}px", (d.y1 - d.y0).abs())
            ></div>
        })
    };
    view! {
        <div
            class="gx-plot"
            node_ref=wrap
            style:height=move || format!("{:.0}px", model.with(|m| m.height))
            data-drag=move || drag.get().map(|d| if d.pan { "pan" } else { "box" })
            on:pointerdown=on_down
            on:pointermove=on_move
            on:pointerup=on_up
            on:pointerleave=on_leave
            on:dblclick=move |_| viewport.set(Viewport::default())
        >
            {svg}
            {ring}
            {brush}
            {tip}
            {move || model.try_with(|m| m.is_empty()).unwrap_or(false).then(|| view! {
                <div class="gx-plot__empty">"No values for this selection"</div>
            })}
        </div>
    }
}
