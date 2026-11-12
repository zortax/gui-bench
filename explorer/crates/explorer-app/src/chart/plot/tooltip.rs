//! The tooltip of an x-y plot: the series under the pointer with its value and interval, or
//! every series at the x of the pointer in crosshair mode.

use explorer_core::units::short;
use leptos::prelude::*;

use super::super::model::{Pt, Series, XyModel};
use super::scene::{Hit, Scene};

fn with_unit(v: f64, unit: &str) -> String {
    if unit.is_empty() {
        short(v)
    } else {
        format!("{} {unit}", short(v))
    }
}

fn interval(name: &str, r: Option<(f64, f64)>, unit: &str) -> Option<(String, String)> {
    r.map(|(lo, hi)| {
        (
            name.to_owned(),
            format!("{} – {}", short(lo), with_unit(hi, unit)),
        )
    })
}

fn point_lines(model: &XyModel, p: &Pt) -> Vec<(String, String)> {
    let mut lines = vec![
        (model.x.label.clone(), model.x.text(p.x)),
        (model.y.label.clone(), with_unit(p.y, &model.y.unit)),
    ];
    lines.extend(interval("band", p.band, &model.y.unit));
    lines.extend(interval("whiskers", p.whisker, &model.y.unit));
    lines.extend(p.tip.iter().cloned());
    lines
}

fn head(s: &Series) -> impl IntoView + use<> {
    view! {
        <div class="gx-tip__head">
            <span class="gx-tip__dot" style:background=s.color.clone()></span>
            <b style:color=s.ink.clone()>{s.label.clone()}</b>
            {s.ghost.then(|| view! { <span class="gx-muted">" (compare run)"</span> })}
        </div>
    }
}

/// The tooltip body for a hit.
pub fn body(model: &XyModel, scene: &Scene, hit: Hit) -> AnyView {
    if hit.bar {
        let Some(b) = model.bars.get(hit.index) else {
            return ().into_any();
        };
        let lines = b.tip.clone();
        return view! {
            <div class="gx-tip__lines">
                {lines.into_iter().map(|(k, v)| view! { <span>{k}</span><b>{v}</b> }).collect_view()}
            </div>
            {b.cell.is_some().then(|| view! { <div class="gx-tip__foot">"click: open the cell"</div> })}
        }
        .into_any();
    }
    if model.crosshair {
        let x = model.series[hit.series].points[hit.index].x;
        let rows: Vec<(&Series, &Pt)> = model
            .series
            .iter()
            .filter_map(|s| s.points.iter().find(|p| p.x == x).map(|p| (s, p)))
            .collect();
        let x_text = model.x.text(x);
        let unit = model.y.unit.clone();
        let _ = scene;
        return view! {
            <div class="gx-tip__head"><b>{format!("{}: {x_text}", model.x.label)}</b></div>
            <div class="gx-tip__lines">
                {rows
                    .into_iter()
                    .map(|(s, p)| {
                        view! {
                            <span style:color=s.ink.clone()>{s.label.clone()}</span>
                            <b>{with_unit(p.y, &unit)}</b>
                        }
                    })
                    .collect_view()}
            </div>
        }
        .into_any();
    }
    let s = &model.series[hit.series];
    let p = &s.points[hit.index];
    let lines = point_lines(model, p);
    view! {
        {head(s)}
        <div class="gx-tip__lines">
            {lines.into_iter().map(|(k, v)| view! { <span>{k}</span><b>{v}</b> }).collect_view()}
        </div>
        {p.invalid.then(|| view! { <div class="gx-tip__warn">"the report fails a validity check"</div> })}
        {p.cell.is_some().then(|| view! { <div class="gx-tip__foot">"click: open the cell"</div> })}
    }
    .into_any()
}
