//! The dashboard at a glance: per framework and variant, the median time to respond to every
//! interaction and the p99 scroll frame, at the small and the large table, colored by the
//! refreshes they take.

use explorer_core::api::{CellInfo, MetricRef};
use explorer_core::units::{short, thousands};
use leptos::prelude::*;

use crate::chart::{ChartCard, HeatCell, HeatModel, Heatmap};
use crate::palette;
use crate::state::{Selection, use_explorer};
use crate::views::kit::Data;

/// The columns: title, measurement, section, metric.
pub const PROBES: [(&str, &str, &str, &str); 5] = [
    ("hover nav", "hover-nav", "latency", "total_ms"),
    ("hover row", "hover-row", "latency", "total_ms"),
    ("compare", "click-compare", "latency", "total_ms"),
    ("filter", "click-filter", "latency", "total_ms"),
    ("scroll p99", "scroll", "frame_pacing", "interval_p99"),
];

/// Returns the color position of a time in refreshes: one refresh is 0, eight are 1.
fn position(refreshes: f64) -> f64 {
    (refreshes.max(1.0).log2() / 3.0).clamp(0.0, 1.0)
}

/// The glance card.
#[component]
pub fn Glance(data: Data, hz: Signal<Option<f64>>) -> impl IntoView {
    view! {
        <ChartCard
            title="Every interaction, both table sizes"
            note="Median input to photon of each interaction, and the p99 frame of the scroll glide · color: refreshes it takes, light is one, dark is eight or more"
            wide=true
        >
            <GlanceMap data=data hz=hz/>
        </ChartCard>
    }
}

#[component]
fn GlanceMap(data: Data, hz: Signal<Option<f64>>) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let sel = Selection::new(data.overview(), &prefs, "dashboard");
        let hz = hz.get().unwrap_or(60.0);
        let cells: Vec<&CellInfo> = sel
            .cells
            .iter()
            .copied()
            .filter(|c| c.key.hz == hz)
            .collect();
        let mut sizes: Vec<u32> = cells.iter().map(|c| c.key.rows).collect();
        sizes.sort_unstable();
        sizes.dedup();
        let mut keys: Vec<(String, String)> = Vec::new();
        for c in &cells {
            let k = (c.key.framework.clone(), c.key.variant.clone());
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        let refresh = 1000.0 / hz;
        let mut cols = Vec::new();
        for (title, ..) in PROBES {
            for rows in &sizes {
                cols.push(format!("{title}\n{} rows", thousands(f64::from(*rows))));
            }
        }
        let grid = keys
            .iter()
            .map(|(fw, variant)| {
                PROBES
                    .iter()
                    .flat_map(|(title, m, s, n)| {
                        let metric = MetricRef::new(m, s, n);
                        let cells = &cells;
                        let data = &data;
                        sizes.iter().map(move |rows| {
                            let c = cells.iter().find(|c| {
                                c.key.framework == *fw
                                    && c.key.variant == *variant
                                    && c.key.rows == *rows
                            })?;
                            let v = data.stat(&metric, c.id)?.median;
                            let invalid = !c.is_valid(m);
                            if invalid && prefs.valid_only {
                                return None;
                            }
                            Some(HeatCell {
                                text: short(v),
                                t: Some(position(v / refresh)),
                                cell: Some(c.id),
                                invalid,
                                tip: vec![
                                    ((*title).to_owned(), format!("{} ms", short(v))),
                                    ("refreshes".to_owned(), short(v / refresh)),
                                    (
                                        "cell".to_owned(),
                                        format!(
                                            "{} · {} rows · {hz} Hz",
                                            c.key.label(),
                                            thousands(f64::from(*rows))
                                        ),
                                    ),
                                ],
                            })
                        })
                    })
                    .collect()
            })
            .collect();
        HeatModel {
            rows: keys
                .iter()
                .map(|(fw, v)| {
                    let label = if v == "default" {
                        fw.clone()
                    } else {
                        format!("{fw} {v}")
                    };
                    (label, palette::ink(fw))
                })
                .collect(),
            cols,
            cells: grid,
            diverging: false,
            ends: (
                "one refresh".to_owned(),
                "eight refreshes or more".to_owned(),
            ),
        }
    });
    view! { <Heatmap model=model/> }
}
