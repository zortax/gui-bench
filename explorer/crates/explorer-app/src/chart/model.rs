//! The data model of an x-y plot: axes, series of points with bands and whiskers, bars and
//! reference rules. Values are in display units; views build models from metric columns.

use explorer_core::api::CellId;
use explorer_core::scale::ScaleKind;

use crate::palette::Marker;

/// An axis of a plot.
#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    /// The default scale; preferences may switch a value axis.
    pub kind: ScaleKind,
    /// The axis title, such as `milliseconds`.
    pub label: String,
    /// The unit after values in tooltips, such as `ms`.
    pub unit: String,
    /// Ticks at fixed values, such as the measured row counts.
    pub fixed: Option<Vec<f64>>,
    /// A linear axis starts at zero.
    pub zero: bool,
    /// The domain, instead of the data range.
    pub domain: Option<(f64, f64)>,
    /// The axis is the value axis that preferences (scale, focus) apply to.
    pub value: bool,
    /// Category labels at fixed positions, with their ink: the rows of a bar chart.
    pub categories: Option<Vec<(f64, String, String)>>,
}

impl Axis {
    /// Returns a linear value axis from zero.
    pub fn value(label: impl Into<String>, unit: impl Into<String>) -> Self {
        Self {
            kind: ScaleKind::Linear,
            label: label.into(),
            unit: unit.into(),
            fixed: None,
            zero: true,
            domain: None,
            value: true,
            categories: None,
        }
    }

    /// Returns a log axis with ticks at fixed values, such as row counts.
    pub fn log_fixed(label: impl Into<String>, ticks: Vec<f64>) -> Self {
        Self {
            kind: ScaleKind::Log,
            label: label.into(),
            unit: String::new(),
            fixed: Some(ticks),
            zero: false,
            domain: None,
            value: false,
            categories: None,
        }
    }

    /// Returns a linear axis over the data, such as time.
    pub fn linear(label: impl Into<String>, unit: impl Into<String>) -> Self {
        Self {
            kind: ScaleKind::Linear,
            label: label.into(),
            unit: unit.into(),
            fixed: None,
            zero: false,
            domain: None,
            value: false,
            categories: None,
        }
    }

    pub fn with_kind(mut self, kind: ScaleKind) -> Self {
        self.kind = kind;
        self
    }

    /// Returns a category axis with labels at 0, 1, 2 … from the top.
    pub fn categories(labels: Vec<(String, String)>) -> Self {
        let n = labels.len() as f64;
        Self {
            kind: ScaleKind::Linear,
            label: String::new(),
            unit: String::new(),
            fixed: None,
            zero: false,
            domain: Some((n - 0.5, -0.5)),
            value: false,
            categories: Some(
                labels
                    .into_iter()
                    .enumerate()
                    .map(|(i, (l, ink))| (i as f64, l, ink))
                    .collect(),
            ),
        }
    }

    /// Returns the text of a value: its category label, or the number with the unit.
    pub fn text(&self, v: f64) -> String {
        if let Some((_, l, _)) = self
            .categories
            .iter()
            .flatten()
            .find(|(c, _, _)| (*c - v).abs() < 1e-9)
        {
            return l.clone();
        }
        let n = explorer_core::units::short(v);
        if self.unit.is_empty() {
            n
        } else {
            format!("{n} {}", self.unit)
        }
    }

    /// Marks the axis as the value axis.
    pub fn valued(mut self) -> Self {
        self.value = true;
        self
    }

    pub fn zero_based(mut self, zero: bool) -> Self {
        self.zero = zero;
        self
    }
}

/// One point of a series.
#[derive(Debug, Clone, PartialEq)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
    pub band: Option<(f64, f64)>,
    pub whisker: Option<(f64, f64)>,
    /// The cell behind the point; a click opens it.
    pub cell: Option<CellId>,
    /// The report behind the point fails a validity check.
    pub invalid: bool,
    /// Extra tooltip lines as (label, value).
    pub tip: Vec<(String, String)>,
}

impl Pt {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            band: None,
            whisker: None,
            cell: None,
            invalid: false,
            tip: Vec::new(),
        }
    }
}

/// A series: a line with markers, bands and whiskers, or markers alone.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    /// A unique key in the plot.
    pub key: String,
    /// The framework, for colors, the legend and highlight.
    pub group: String,
    /// The name in tooltips and end labels.
    pub label: String,
    /// The line color, a CSS color.
    pub color: String,
    /// The band and whisker color.
    pub soft: String,
    /// The label color.
    pub ink: String,
    /// The dash pattern of the line.
    pub dash: String,
    pub marker: Marker,
    pub hollow: bool,
    /// The series belongs to the compare run: drawn faint.
    pub ghost: bool,
    /// Points connect with a line.
    pub line: bool,
    /// The line steps (ECDF, timelines).
    pub step: bool,
    /// The marker radius; zero draws no markers.
    pub radius: f64,
    /// The series gets a direct label at its last point.
    pub end_label: bool,
    /// A pixel offset of the x position, so whiskers at one x stay apart.
    pub dodge: f64,
    pub points: Vec<Pt>,
}

impl Series {
    /// Returns a line series in the colors of a framework.
    pub fn of_framework(key: impl Into<String>, framework: &str, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            group: framework.to_owned(),
            label: label.into(),
            color: crate::palette::mark(framework),
            soft: crate::palette::soft(framework),
            ink: crate::palette::ink(framework),
            dash: String::new(),
            marker: Marker::Circle,
            hollow: false,
            ghost: false,
            line: true,
            step: false,
            radius: 3.2,
            end_label: false,
            dodge: 0.0,
            points: Vec::new(),
        }
    }
}

/// A bar from (x0, y0) to (x1, y1), such as a histogram bin or a stacked segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
    pub color: String,
    pub group: String,
    pub tip: Vec<(String, String)>,
    pub cell: Option<CellId>,
}

/// A reference line across the plot, such as one refresh period.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// The rule is horizontal (a y value); otherwise vertical.
    pub horizontal: bool,
    pub value: f64,
    pub label: String,
}

/// An x-y plot.
#[derive(Debug, Clone, PartialEq)]
pub struct XyModel {
    pub x: Axis,
    pub y: Axis,
    pub series: Vec<Series>,
    pub bars: Vec<Bar>,
    pub rules: Vec<Rule>,
    /// The plot height in pixels, without axes.
    pub height: f64,
    /// The tooltip snaps to the x of the pointer and lists every series there.
    pub crosshair: bool,
}

impl XyModel {
    pub fn new(x: Axis, y: Axis) -> Self {
        Self {
            x,
            y,
            series: Vec::new(),
            bars: Vec::new(),
            rules: Vec::new(),
            height: 280.0,
            crosshair: false,
        }
    }

    /// Returns true without any point or bar.
    pub fn is_empty(&self) -> bool {
        self.series.iter().all(|s| s.points.is_empty()) && self.bars.is_empty()
    }
}
