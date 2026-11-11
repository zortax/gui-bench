//! The SVG layers of an x-y plot, drawn from a scene: the grid and axes, reference rules, bars,
//! series (bands, whiskers, lines, markers), the pins of values outside the focus range, and the
//! direct labels at line ends.

use explorer_core::scale::{ScaleKind, spread};
use explorer_core::units::short;

use crate::palette::Marker;
use leptos::prelude::*;

use super::super::model::{Series, XyModel};
use super::scene::Scene;

fn f(v: f64) -> String {
    format!("{v:.1}")
}

/// The grid on the value axis, the hairline base, tick labels and axis titles.
pub fn axes(scene: &Scene, model: &XyModel) -> impl IntoView + use<> {
    let s = scene.clone();
    let cats = model.y.categories.clone();
    let grid = s
        .y_ticks
        .iter()
        .enumerate()
        .map(|(i, (v, l, major))| {
            let y = f(s.y.map(*v));
            let ink = cats.as_ref().and_then(|c| c.get(i)).map(|(_, _, ink)| ink.clone());
            let line = cats.is_none().then(|| view! {
                <line class="gx-gridline" data-minor=(!major).then_some("") x1=f(s.left) x2=f(s.right) y1=y.clone() y2=y.clone()/>
            });
            let text = (!l.is_empty()).then(|| view! {
                <text class="gx-tick" x=f(s.left - 7.0) y=y text-anchor="end" dominant-baseline="middle" style:fill=ink>{l.clone()}</text>
            });
            (line, text)
        })
        .collect_view();
    let x_grid = (cats.is_some() || model.x.value).then(|| {
        s.x_ticks
            .iter()
            .map(|(v, _, major)| {
                let x = f(s.x.map(*v));
                view! { <line class="gx-gridline" data-minor=(!major).then_some("") x1=x.clone() x2=x y1=f(s.top) y2=f(s.bottom)/> }
            })
            .collect_view()
    });
    let x_ticks = s
        .x_ticks
        .iter()
        .filter(|(_, l, _)| !l.is_empty())
        .map(|(v, l, _)| {
            view! {
                <text class="gx-tick" x=f(s.x.map(*v)) y=f(s.bottom + 17.0) text-anchor="middle">{l.clone()}</text>
            }
        })
        .collect_view();
    let x_label = model.x.label.clone();
    let y_label = model.y.label.clone();
    let mid_x = f((s.left + s.right) / 2.0);
    let mid_y = (s.top + s.bottom) / 2.0;
    view! {
        <g class="gx-axes">
            {x_grid}
            {grid}
            <line class="gx-base" x1=f(s.left) x2=f(s.right) y1=f(s.bottom) y2=f(s.bottom)/>
            {x_ticks}
            <text class="gx-axis-title" x=mid_x y=f(s.bottom + 33.0) text-anchor="middle">{x_label}</text>
            <text
                class="gx-axis-title"
                x="0"
                y="0"
                text-anchor="middle"
                transform=format!("translate(12 {mid_y:.1}) rotate(-90)")
            >
                {y_label}
            </text>
        </g>
    }
}

/// Reference rules, such as one refresh period.
pub fn rules(scene: &Scene, model: &XyModel) -> impl IntoView + use<> {
    let s = scene.clone();
    model
        .rules
        .iter()
        .filter(|r| {
            if r.horizontal {
                s.y.contains(r.value)
            } else {
                s.x.contains(r.value)
            }
        })
        .map(|r| {
            if r.horizontal {
                let y = s.y.map(r.value);
                view! {
                    <g class="gx-rule">
                        <line x1=f(s.left) x2=f(s.right) y1=f(y) y2=f(y)/>
                        <text x=f(s.left + 4.0) y=f(y - 4.0)>{r.label.clone()}</text>
                    </g>
                }
            } else {
                let x = s.x.map(r.value);
                view! {
                    <g class="gx-rule">
                        <line x1=f(x) x2=f(x) y1=f(s.top) y2=f(s.bottom)/>
                        <text x=f(x + 4.0) y=f(s.top + 9.0)>{r.label.clone()}</text>
                    </g>
                }
            }
        })
        .collect_view()
}

/// Bars, each with a 1 px gap to its neighbours.
pub fn bars(
    scene: &Scene,
    model: &XyModel,
    highlight: RwSignal<Option<String>>,
) -> impl IntoView + use<> {
    let s = scene.clone();
    model
        .bars
        .iter()
        .map(|b| {
            let x0 = s.x.map(b.x0);
            let x1 = s.x.map(b.x1);
            let y0 = s.y.map(b.y0);
            let y1 = s.y.map(b.y1);
            let group = b.group.clone();
            let dim = move || {
                highlight
                    .with(|h| h.as_ref().is_some_and(|h| *h != group))
                    .then_some("")
            };
            view! {
                <rect
                    class="gx-bar"
                    data-dim=dim
                    x=f(x0.min(x1) + 0.5)
                    y=f(y0.min(y1))
                    width=f(((x1 - x0).abs() - 1.0).max(0.8))
                    height=f((y1 - y0).abs().max(0.5))
                    style:fill=b.color.clone()
                />
            }
        })
        .collect_view()
}

/// Returns true when a point lies where its log axes can show it.
fn shown(scene: &Scene, x: f64, y: f64) -> bool {
    (scene.x.kind != ScaleKind::Log || x > 0.0) && (scene.y.kind != ScaleKind::Log || y > 0.0)
}

fn line_path(scene: &Scene, s: &Series) -> String {
    let mut d = String::new();
    let mut prev: Option<(f64, f64)> = None;
    for p in &s.points {
        if (scene.x.kind == ScaleKind::Log && p.x <= 0.0)
            || (scene.y.kind == ScaleKind::Log && p.y <= 0.0)
        {
            prev = None;
            continue;
        }
        let x = scene.x.map(p.x) + s.dodge;
        let y = scene.y.map(p.y);
        match prev {
            None => d.push_str(&format!("M{x:.1},{y:.1}")),
            Some(_) if s.step => d.push_str(&format!("H{x:.1}V{y:.1}")),
            Some(_) => d.push_str(&format!("L{x:.1},{y:.1}")),
        }
        prev = Some((x, y));
    }
    d
}

fn band_path(scene: &Scene, s: &Series) -> Option<String> {
    let pts: Vec<_> = s
        .points
        .iter()
        .filter_map(|p| p.band.map(|b| (p.x, b)))
        .collect();
    if pts.len() < 2 || !s.line {
        return None;
    }
    let mut d = String::new();
    for (i, (x, (_, hi))) in pts.iter().enumerate() {
        let px = scene.x.map(*x) + s.dodge;
        d.push_str(&format!(
            "{}{px:.1},{:.1}",
            if i == 0 { "M" } else { "L" },
            scene.y.map(*hi)
        ));
    }
    for (x, (lo, _)) in pts.iter().rev() {
        d.push_str(&format!(
            "L{:.1},{:.1}",
            scene.x.map(*x) + s.dodge,
            scene.y.map(*lo)
        ));
    }
    d.push('Z');
    Some(d)
}

/// Every series: bands, whiskers, the line and the markers.
pub fn series(
    scene: &Scene,
    model: &XyModel,
    highlight: RwSignal<Option<String>>,
) -> impl IntoView + use<> {
    let sc = scene.clone();
    model
        .series
        .iter()
        .map(|s| {
            let group = s.group.clone();
            let dim = move || highlight.with(|h| h.as_ref().is_some_and(|h| *h != group)).then_some("");
            let band = band_path(&sc, s).map(|d| view! { <path class="gx-band" d=d/> });
            let ranges = s
                .points
                .iter()
                .filter(|p| !(sc.x.kind == ScaleKind::Log && p.x <= 0.0))
                .map(|p| {
                    let x = f(sc.x.map(p.x) + s.dodge);
                    let whisker = p.whisker.map(|(lo, hi)| view! {
                        <line class="gx-whisker" x1=x.clone() x2=x.clone() y1=f(sc.y.map(lo)) y2=f(sc.y.map(hi))/>
                    });
                    let thick = (!s.line).then_some(p.band).flatten().map(|(lo, hi)| view! {
                        <line class="gx-iqr" x1=x.clone() x2=x.clone() y1=f(sc.y.map(lo)) y2=f(sc.y.map(hi))/>
                    });
                    (whisker, thick)
                })
                .collect_view();
            let line = s.line.then(|| view! {
                <path class="gx-line" d=line_path(&sc, s) stroke-dasharray=(!s.dash.is_empty()).then(|| s.dash.clone())/>
            });
            let markers = (s.radius > 0.0).then(|| {
                s.points
                    .iter()
                    .filter(|p| shown(&sc, p.x, p.y))
                    .map(|p| {
                        let hollow = (s.hollow || s.ghost || p.invalid).then_some("");
                        let invalid = p.invalid.then_some("");
                        let x = sc.x.map(p.x) + s.dodge;
                        let y = sc.y.map(p.y);
                        if s.marker == Marker::Circle {
                            view! { <circle class="gx-marker" data-hollow=hollow data-invalid=invalid cx=f(x) cy=f(y) r=f(s.radius)/> }.into_any()
                        } else {
                            view! {
                                <path
                                    class="gx-marker"
                                    data-hollow=hollow
                                    data-invalid=invalid
                                    d=s.marker.path(s.radius)
                                    transform=format!("translate({x:.1} {y:.1})")
                                />
                            }
                            .into_any()
                        }
                    })
                    .collect_view()
            });
            view! {
                <g class="gx-series" data-dim=dim data-ghost=s.ghost.then_some("") style:--c=s.color.clone() style:--s=s.soft.clone()>
                    {band}
                    {ranges}
                    {line}
                    {markers}
                </g>
            }
        })
        .collect_view()
}

/// Pins for values above the focus range: a chevron at the top edge with the value.
pub fn pins(scene: &Scene, model: &XyModel) -> impl IntoView + use<> {
    if !scene.focused {
        return ().into_any();
    }
    let (_, top_value) = scene.y.bounds();
    let mut pins: Vec<(f64, String, String)> = Vec::new();
    for s in model.series.iter().filter(|s| !s.ghost) {
        for p in &s.points {
            if p.y > top_value && scene.x.contains(p.x) {
                pins.push((scene.x.map(p.x) + s.dodge, short(p.y), s.ink.clone()));
            }
        }
    }
    pins.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut last_x = f64::NEG_INFINITY;
    let mut stack = 0.0;
    pins.into_iter()
        .map(|(x, value, ink)| {
            stack = if (x - last_x).abs() < 30.0 {
                stack + 11.0
            } else {
                0.0
            };
            last_x = x;
            let y = scene.top + 2.0;
            view! {
                <g class="gx-pin" style:fill=ink.clone()>
                    <path d=format!("M{:.1},{:.1}l4,-5l4,5z", x - 4.0, y + 4.0)/>
                    <text x=f(x + 6.0) y=f(y + 3.0 + stack)>{value}</text>
                </g>
            }
        })
        .collect_view()
        .into_any()
}

/// Direct labels at the last point of labelled series, kept apart.
pub fn end_labels(scene: &Scene, model: &XyModel, unit: &str) -> impl IntoView + use<> {
    let (_, top_value) = scene.y.bounds();
    let items: Vec<(f64, &Series, f64)> = model
        .series
        .iter()
        .filter(|s| s.end_label && !s.ghost)
        .filter_map(|s| {
            let p = s
                .points
                .iter()
                .filter(|p| scene.x.contains(p.x))
                .max_by(|a, b| a.x.total_cmp(&b.x))?;
            let y = if p.y > top_value {
                scene.top
            } else {
                scene.y.map(p.y)
            };
            Some((y, s, p.y))
        })
        .collect();
    let wants: Vec<f64> = items.iter().map(|(y, _, _)| *y).collect();
    let got = spread(&wants, 14.0);
    let x = scene.right + 9.0;
    let unit = unit.to_owned();
    items
        .into_iter()
        .zip(got)
        .map(|((_, s, value), y)| {
            let u = unit.clone();
            view! {
                <text class="gx-endlabel" x=f(x) y=f(y) dominant-baseline="middle">
                    <tspan class="gx-endlabel__name" style:fill=s.ink.clone()>{s.label.clone()}</tspan>
                    <tspan class="gx-endlabel__value">{format!(" {}{}{u}", short(value), if u.is_empty() { "" } else { " " })}</tspan>
                </text>
            }
        })
        .collect_view()
}
