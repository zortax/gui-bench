//! The scene of an x-y plot: the plot rectangle, the scales after preferences and zoom, the
//! ticks, and the hit targets of points and bars.

use explorer_core::scale::{Scale, ScaleKind, Tick, padded};
use explorer_core::stats::focus_limit;
use explorer_core::units::thousands;

use super::super::interact::Viewport;
use super::super::model::XyModel;

/// The pixel width of one tick label character.
const CHAR: f64 = 6.8;

/// A hit target: a point or a bar centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub px: f64,
    pub py: f64,
    pub series: usize,
    /// The point index, or the bar index when `bar` is set.
    pub index: usize,
    pub bar: bool,
}

/// The geometry of a plot at one size and zoom.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub width: f64,
    pub height: f64,
    /// The plot rectangle: left, top, right, bottom.
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub x: Scale,
    pub y: Scale,
    pub x_ticks: Vec<(f64, String, bool)>,
    pub y_ticks: Vec<(f64, String, bool)>,
    /// The focus mode cut the value axis.
    pub focused: bool,
    /// The hit targets, shared so a scene clones cheaply.
    pub hits: std::sync::Arc<Vec<Hit>>,
}

fn label(t: &Tick) -> String {
    if t.major {
        thousands(t.value)
    } else {
        String::new()
    }
}

/// Returns the data range of the y values of a model: values, bands, whiskers and bars.
fn y_range(model: &XyModel, log: bool) -> Option<(f64, f64)> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut take = |v: f64| {
        if v.is_finite() && (!log || v > 0.0) {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    };
    for s in &model.series {
        for p in &s.points {
            take(p.y);
            if let Some((a, b)) = p.band {
                take(a);
                take(b);
            }
            if let Some((a, b)) = p.whisker {
                take(a);
                take(b);
            }
        }
    }
    for b in &model.bars {
        take(b.y0);
        take(b.y1);
    }
    (lo <= hi).then_some((lo, hi))
}

fn x_range(model: &XyModel, log: bool) -> Option<(f64, f64)> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut take = |v: f64| {
        if v.is_finite() && (!log || v > 0.0) {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    };
    for s in &model.series {
        for p in &s.points {
            take(p.x);
        }
    }
    for b in &model.bars {
        take(b.x0);
        take(b.x1);
    }
    if let Some(fixed) = &model.x.fixed {
        fixed.iter().copied().for_each(&mut take);
    }
    (lo <= hi).then_some((lo, hi))
}

/// Returns the upper bound of the value axis in focus mode: the peaks of every series (and
/// bar), with the outlying ones cut off.
fn focus_top(model: &XyModel) -> Option<f64> {
    let mut peaks: Vec<f64> = model
        .series
        .iter()
        .filter(|s| !s.ghost)
        .filter_map(|s| {
            s.points
                .iter()
                .map(|p| p.y)
                .fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| m.max(v))))
        })
        .collect();
    if model.series.len() < 3 {
        peaks = model
            .series
            .iter()
            .flat_map(|s| s.points.iter().map(|p| p.y))
            .collect();
    }
    peaks.extend(model.bars.iter().map(|b| b.y1));
    focus_limit(&peaks)
}

/// The inputs of a scene besides the model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneOpts {
    pub width: f64,
    pub x_kind: ScaleKind,
    pub y_kind: ScaleKind,
    pub focus: bool,
    pub view: Viewport,
    /// The pixel width reserved right of the plot for end labels.
    pub label_room: f64,
}

/// Lays out a plot.
pub fn layout(model: &XyModel, o: SceneOpts) -> Scene {
    let y_log = o.y_kind == ScaleKind::Log;
    let x_log = o.x_kind == ScaleKind::Log;
    let (ylo, yhi) = y_range(model, y_log).unwrap_or((0.0, 1.0));
    let mut focused = false;
    let mut y_dom = model.y.domain.unwrap_or_else(|| {
        let hi = if o.focus && model.y.value {
            match focus_top(model) {
                Some(top) if top < yhi => {
                    focused = true;
                    top
                }
                _ => yhi,
            }
        } else {
            yhi
        };
        let lo = if y_log {
            ylo
        } else {
            ylo.min(if model.y.zero { 0.0 } else { ylo })
        };
        padded(o.y_kind, lo, hi, model.y.zero && !y_log)
    });
    if let Some(v) = o.view.y {
        y_dom = v;
    }
    let (xlo, xhi) = x_range(model, x_log).unwrap_or((0.0, 1.0));
    let mut x_dom = model.x.domain.unwrap_or_else(|| {
        if x_log {
            padded(ScaleKind::Log, xlo, xhi, false)
        } else if model.x.value {
            padded(ScaleKind::Linear, xlo, xhi, model.x.zero)
        } else {
            let span = (xhi - xlo).abs().max(f64::EPSILON);
            let pad = if model.bars.is_empty() {
                span * 0.03
            } else {
                0.0
            };
            (
                if model.x.zero {
                    xlo.min(0.0)
                } else {
                    xlo - pad
                },
                xhi + pad,
            )
        }
    });
    if let Some(v) = o.view.x {
        x_dom = v;
    }
    let height = model.height;
    let top = 12.0;
    let bottom = height - 38.0;
    let probe = Scale::new(o.y_kind, y_dom, (bottom, top));
    let y_count = ((bottom - top) / 42.0).round().max(2.0) as usize;
    let y_ticks: Vec<(f64, String, bool)> = match &model.y.categories {
        Some(cats) => cats.iter().map(|(v, l, _)| (*v, l.clone(), true)).collect(),
        None => probe
            .ticks(y_count)
            .iter()
            .map(|t| (t.value, label(t), t.major))
            .collect(),
    };
    let widest = y_ticks.iter().map(|(_, l, _)| l.len()).max().unwrap_or(1) as f64;
    let left = (widest * CHAR + 26.0).max(38.0);
    let right = (o.width - o.label_room - 10.0).max(left + 40.0);
    let x = Scale::new(o.x_kind, x_dom, (left, right));
    let y = Scale::new(o.y_kind, y_dom, (bottom, top));
    let x_ticks: Vec<(f64, String, bool)> = match &model.x.fixed {
        Some(fixed) if o.view.x.is_none() => fixed
            .iter()
            .filter(|v| x.contains(**v))
            .map(|v| (*v, thousands(*v), true))
            .collect(),
        _ => {
            let count = ((right - left) / 80.0).round().max(2.0) as usize;
            x.ticks(count)
                .iter()
                .map(|t| (t.value, label(t), t.major))
                .collect()
        }
    };
    let mut hits = Vec::new();
    for (si, s) in model.series.iter().enumerate() {
        for (pi, p) in s.points.iter().enumerate() {
            if (x_log && p.x <= 0.0) || (y_log && p.y <= 0.0) {
                continue;
            }
            let py = y.map(p.y).clamp(top - 4.0, bottom + 4.0);
            hits.push(Hit {
                px: x.map(p.x) + s.dodge,
                py,
                series: si,
                index: pi,
                bar: false,
            });
        }
    }
    for (bi, b) in model.bars.iter().enumerate() {
        hits.push(Hit {
            px: (x.map(b.x0) + x.map(b.x1)) / 2.0,
            py: y.map((b.y0 + b.y1) / 2.0),
            series: usize::MAX,
            index: bi,
            bar: true,
        });
    }
    Scene {
        width: o.width,
        height,
        left,
        top,
        right,
        bottom,
        x,
        y,
        x_ticks,
        y_ticks,
        focused,
        hits: std::sync::Arc::new(hits),
    }
}

impl Scene {
    /// Returns the hit nearest to a position within `reach` pixels.
    pub fn nearest(&self, px: f64, py: f64, reach: f64) -> Option<Hit> {
        self.hits
            .iter()
            .map(|h| (h, (h.px - px).powi(2) + (h.py - py).powi(2)))
            .filter(|(_, d)| *d <= reach * reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(h, _)| *h)
    }

    /// Returns the hit with the x nearest to a position, for crosshair tooltips.
    pub fn nearest_x(&self, px: f64) -> Option<Hit> {
        self.hits
            .iter()
            .filter(|h| !h.bar)
            .min_by(|a, b| (a.px - px).abs().total_cmp(&(b.px - px).abs()))
            .copied()
    }

    /// Returns true when a position lies inside the plot rectangle.
    pub fn inside(&self, px: f64, py: f64) -> bool {
        px >= self.left && px <= self.right && py >= self.top && py <= self.bottom
    }
}
