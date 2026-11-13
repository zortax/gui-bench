//! The preferences of the explorer, read from the URL query: framework and refresh rate
//! filters, the value scale, the uncertainty shown, outlier focus, the baseline framework,
//! the compare run, the valid-only filter and the open cell.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

/// The value scale of charts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalePref {
    /// Each chart picks its own scale.
    Auto,
    Linear,
    Log,
}

impl ScalePref {
    pub fn parse(text: &str) -> Self {
        match text {
            "lin" => Self::Linear,
            "log" => Self::Log,
            _ => Self::Auto,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "",
            Self::Linear => "lin",
            Self::Log => "log",
        }
    }
}

/// The interval that whiskers and bands show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Uncertainty {
    /// Samples: the middle half as a band and p5 to p95 as whiskers. Iterations: the range.
    Auto,
    /// The 95 % confidence interval of the report.
    Ci,
    /// The middle half.
    Iqr,
    /// The 5th to 95th percentile.
    P90,
    /// The smallest to the largest value.
    Range,
    None,
}

impl Uncertainty {
    pub const ALL: [Self; 6] = [
        Self::Auto,
        Self::Ci,
        Self::Iqr,
        Self::P90,
        Self::Range,
        Self::None,
    ];

    pub fn parse(text: &str) -> Self {
        match text {
            "ci" => Self::Ci,
            "iqr" => Self::Iqr,
            "p90" => Self::P90,
            "range" => Self::Range,
            "none" => Self::None,
            _ => Self::Auto,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "",
            Self::Ci => "ci",
            Self::Iqr => "iqr",
            Self::P90 => "p90",
            Self::Range => "range",
            Self::None => "none",
        }
    }

    /// Returns the short label of a control.
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Ci => "95% CI",
            Self::Iqr => "IQR",
            Self::P90 => "p5–95",
            Self::Range => "range",
            Self::None => "none",
        }
    }
}

/// The preferences as memos over the URL query.
#[derive(Clone, Copy)]
pub struct Prefs {
    pub hidden: Memo<Vec<String>>,
    pub rates: Memo<Vec<String>>,
    pub scale: Memo<ScalePref>,
    pub uncertainty: Memo<Uncertainty>,
    pub focus: Memo<bool>,
    pub base: Memo<Option<String>>,
    pub compare: Memo<Option<String>>,
    pub valid_only: Memo<bool>,
    pub cell: Memo<Option<u32>>,
}

/// The preferences at one moment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefsSnapshot {
    pub hidden: Vec<String>,
    pub rates: Vec<String>,
    pub scale: ScalePref,
    pub uncertainty: Uncertainty,
    pub focus: bool,
    pub base: Option<String>,
    pub compare: Option<String>,
    pub valid_only: bool,
}

fn list(text: Option<String>) -> Vec<String> {
    text.map(|t| {
        t.split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    })
    .unwrap_or_default()
}

impl Prefs {
    /// Returns the preferences of the current URL.
    pub fn from_query() -> Self {
        let q = use_query_map();
        let get = move |key: &'static str| q.with(|q| q.get(key));
        Self {
            hidden: Memo::new(move |_| list(get("hide"))),
            rates: Memo::new(move |_| list(get("hz"))),
            scale: Memo::new(move |_| ScalePref::parse(&get("scale").unwrap_or_default())),
            uncertainty: Memo::new(move |_| Uncertainty::parse(&get("unc").unwrap_or_default())),
            focus: Memo::new(move |_| get("focus").is_some_and(|v| v == "1")),
            base: Memo::new(move |_| get("base")),
            compare: Memo::new(move |_| get("cmp")),
            valid_only: Memo::new(move |_| get("valid").is_some_and(|v| v == "1")),
            cell: Memo::new(move |_| get("cell").and_then(|c| c.parse().ok())),
        }
    }

    /// Returns the preferences at this moment, tracking every one.
    pub fn snapshot(&self) -> PrefsSnapshot {
        PrefsSnapshot {
            hidden: self.hidden.get(),
            rates: self.rates.get(),
            scale: self.scale.get(),
            uncertainty: self.uncertainty.get(),
            focus: self.focus.get(),
            base: self.base.get(),
            compare: self.compare.get(),
            valid_only: self.valid_only.get(),
        }
    }
}

impl PrefsSnapshot {
    /// Returns true when a framework shows.
    pub fn shows_framework(&self, framework: &str) -> bool {
        !self.hidden.iter().any(|h| h == framework)
    }

    /// Returns true when a refresh rate shows.
    pub fn shows_rate(&self, hz: f64) -> bool {
        self.rates.is_empty() || self.rates.iter().any(|r| r.parse::<f64>().ok() == Some(hz))
    }
}
