//! The framework colors and order of the gui-bench plots. A framework takes a fixed Oldworld hue
//! by its place in [`FRAMEWORKS`], so it keeps its color whatever a run or a filter selects. An
//! edition of a framework (another build of the same app) takes the hue one step deeper and
//! sorts right after it. The colors themselves live in the style sheet as CSS variables per slot
//! and theme; this module names the slot.

/// The frameworks in plot order. Their index picks the hue.
pub const FRAMEWORKS: [&str; 8] = [
    "zgui", "gpui-kit", "freya", "vizia", "floem", "iced", "egui", "dioxus",
];
/// Editions and their framework.
pub const EDITIONS: [(&str, &str); 5] = [
    ("gpui-kit-tuned", "gpui-kit"),
    ("gpui-kit-fast", "gpui-kit"),
    ("iced-tuned", "iced"),
    ("freya-056a5b4", "freya"),
    ("dioxus-skia", "dioxus"),
];
/// The number of hues.
const HUES: usize = 8;

/// Returns the framework of an edition, or the name itself.
pub fn base(framework: &str) -> &str {
    EDITIONS
        .iter()
        .find(|(e, _)| *e == framework)
        .map_or(framework, |(_, b)| b)
}

/// Returns true when the framework is an edition of another.
pub fn is_edition(framework: &str) -> bool {
    EDITIONS.iter().any(|(e, _)| *e == framework)
}

/// Returns the color slot of a framework, such as `3` or `1e`.
pub fn slot(framework: &str) -> String {
    let b = base(framework);
    let index = FRAMEWORKS
        .iter()
        .position(|f| *f == b)
        .unwrap_or_else(|| FRAMEWORKS.len() + (b.bytes().map(usize::from).sum::<usize>() % HUES));
    let index = index % HUES;
    if is_edition(framework) {
        format!("{index}e")
    } else {
        index.to_string()
    }
}

/// Returns the mid tone of a framework as a CSS color: lines and marks.
pub fn mark(framework: &str) -> String {
    format!("var(--fw-{}-mark)", slot(framework))
}

/// Returns the pastel of a framework as a CSS color: bands, ranges and fills.
pub fn soft(framework: &str) -> String {
    format!("var(--fw-{}-soft)", slot(framework))
}

/// Returns the ink of a framework as a CSS color: its name and values as text.
pub fn ink(framework: &str) -> String {
    format!("var(--fw-{}-ink)", slot(framework))
}

/// Returns the place of a framework in plots: editions right after their framework.
pub fn rank(framework: &str) -> f64 {
    let b = base(framework);
    let r = FRAMEWORKS
        .iter()
        .position(|f| *f == b)
        .unwrap_or(FRAMEWORKS.len()) as f64;
    r + if is_edition(framework) { 0.5 } else { 0.0 }
}

/// Sorts frameworks in plot order, then by name.
pub fn sort(frameworks: &mut [String]) {
    frameworks.sort_by(|a, b| rank(a).total_cmp(&rank(b)).then_with(|| a.cmp(b)));
}

/// The marker of a series, a second encoding on top of the framework color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    Circle,
    Diamond,
    Square,
    /// A cross: a size that did not run.
    Cross,
}

impl Marker {
    /// Returns an SVG path of the marker centred on the origin with radius `r`.
    pub fn path(self, r: f64) -> String {
        match self {
            Self::Circle => format!(
                "M{:.2},0A{r:.2},{r:.2} 0 1,0 {:.2},0A{r:.2},{r:.2} 0 1,0 {:.2},0Z",
                -r, r, -r
            ),
            Self::Square => {
                let s = r * 0.88;
                format!("M{:.2},{:.2}H{s:.2}V{s:.2}H{:.2}Z", -s, -s, -s)
            }
            Self::Diamond => {
                let s = r * 1.2;
                format!("M0,{:.2}L{s:.2},0L0,{s:.2}L{:.2},0Z", -s, -s)
            }
            Self::Cross => {
                let (a, b) = (r * 1.25, r * 0.3);
                format!(
                    "M{:.2},{:.2}L{:.2},{:.2}L0,{:.2}L{:.2},{:.2}L{:.2},{:.2}L{:.2},0L{:.2},{:.2}L{:.2},{:.2}L0,{:.2}L{:.2},{:.2}L{:.2},{:.2}L{:.2},0Z",
                    -a,
                    -a + b,
                    -a + b,
                    -a,
                    -b,
                    a - b,
                    -a,
                    a,
                    -a + b,
                    b,
                    a,
                    a - b,
                    a - b,
                    a,
                    b,
                    -a + b,
                    a,
                    -a,
                    a - b,
                    -b
                )
            }
        }
    }
}

/// The dash pattern of the line of a framework: editions are dashed.
pub fn dash(framework: &str) -> &'static str {
    if is_edition(framework) { "5 3" } else { "" }
}
