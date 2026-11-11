//! The efficiency matrix: the app CPU time and GPU time per second of the steady phase of every
//! scenario, one row per framework, colored on a log scale per matrix.

use explorer_core::api::{CellInfo, MetricRef};
use explorer_core::units::short;
use leptos::prelude::*;

use crate::chart::{ChartCard, HeatCell, HeatModel, Heatmap};
use crate::palette;
use crate::state::{Selection, use_explorer};
use crate::views::kit::Data;
use crate::widgets::Segmented;

/// The columns: label, scenario, measurement, rows, updates.
const COLUMNS: [(&str, &str, &str, u32, Option<u32>); 10] = [
    ("idle\n1k rows", "rows", "idle", 1000, None),
    ("hover\n1k rows", "rows", "hover", 1000, None),
    ("scroll\n1k rows", "rows", "scroll", 1000, None),
    ("scroll\n10k rows", "rows", "scroll", 10000, None),
    ("ticker 1k\n1/tick", "ticker", "pacing", 1000, Some(1)),
    ("ticker 10k\n100/tick", "ticker", "pacing", 10000, Some(100)),
    ("spinner\nno rows", "spinner", "pacing", 0, None),
    ("spinner\n10k rows", "spinner", "pacing", 10000, None),
    ("typing\n1k rows", "typing", "typing", 1000, None),
    ("scroll\n1M virtual", "virtual", "scroll", 1_000_000, None),
];

fn metric(measurement: &str, gpu: bool) -> MetricRef {
    if gpu {
        MetricRef::new(measurement, "frame_cost", "gpu_fence_ms_per_s")
    } else {
        MetricRef::new(measurement, "resources", "cpu_ms_per_s")
    }
}

/// The metrics the matrix reads.
pub fn metrics() -> Vec<MetricRef> {
    let mut out = Vec::new();
    for (_, _, m, _, _) in COLUMNS {
        for gpu in [false, true] {
            let r = metric(m, gpu);
            if !out.contains(&r) {
                out.push(r);
            }
        }
    }
    out
}

/// The efficiency matrix.
#[component]
pub fn Efficiency(data: Data) -> impl IntoView {
    let what = RwSignal::new("cpu".to_owned());
    let controls = ViewFn::from(move || {
        view! {
            <Segmented
                options=vec![("cpu".to_owned(), "CPU ms/s".to_owned()), ("gpu".to_owned(), "GPU ms/s".to_owned())]
                value=Signal::derive(move || what.get())
                on_change=Callback::new(move |v| what.set(v))
            />
        }
    });
    view! {
        <ChartCard
            title="Power proxy: time per second of every scenario"
            note="The steady phase of each scenario at the lowest refresh rate, the finest variant · darker is costlier, on a log scale"
            controls=controls
            wide=true
        >
            <Matrix data=data what=what/>
        </ChartCard>
    }
}

#[component]
fn Matrix(data: Data, what: RwSignal<String>) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let gpu = what.get() == "gpu";
        let mut fws: Vec<String> = Vec::new();
        let mut all: Vec<&CellInfo> = Vec::new();
        for (_, sc, ..) in COLUMNS {
            let sel = Selection::new(data.overview(), &prefs, sc);
            for f in sel.frameworks() {
                if !fws.contains(&f) {
                    fws.push(f);
                }
            }
            all.extend(sel.cells);
        }
        palette::sort(&mut fws);
        let pick = |fw: &str, (sc, rows, updates): (&str, u32, Option<u32>)| {
            all.iter()
                .copied()
                .filter(|c| {
                    c.key.framework == fw
                        && c.key.scenario == sc
                        && c.key.rows == rows
                        && c.key.updates == updates
                })
                .min_by(|a, b| {
                    a.key.hz.total_cmp(&b.key.hz).then(
                        palette::variant_rank(&a.key.variant)
                            .cmp(&palette::variant_rank(&b.key.variant)),
                    )
                })
        };
        let cols: Vec<_> = COLUMNS
            .iter()
            .filter(|(_, sc, m, r, u)| {
                fws.iter().any(|f| {
                    pick(f, (sc, *r, *u))
                        .and_then(|c| data.stat(&metric(m, gpu), c.id))
                        .is_some()
                })
            })
            .collect();
        let values: Vec<Vec<Option<(f64, &CellInfo)>>> = fws
            .iter()
            .map(|f| {
                cols.iter()
                    .map(|(_, sc, m, r, u)| {
                        let c = pick(f, (sc, *r, *u))?;
                        Some((data.stat(&metric(m, gpu), c.id)?.median, c))
                    })
                    .collect()
            })
            .collect();
        let top = values
            .iter()
            .flatten()
            .flatten()
            .map(|(v, _)| *v)
            .fold(0.2, f64::max);
        let floor = (top / 1000.0).max(0.1);
        let t = |v: f64| ((v.max(floor) / floor).ln() / (top / floor).ln()).clamp(0.0, 1.0);
        HeatModel {
            rows: fws.iter().map(|f| (f.clone(), palette::ink(f))).collect(),
            cols: cols.iter().map(|(l, ..)| (*l).to_owned()).collect(),
            cells: values
                .iter()
                .map(|row| {
                    row.iter()
                        .zip(&cols)
                        .map(|(v, (_, _, m, _, _))| {
                            v.map(|(v, c)| HeatCell {
                                text: short(v),
                                t: Some(t(v)),
                                cell: Some(c.id),
                                invalid: !c.is_valid(m),
                                tip: vec![
                                    (
                                        if gpu { "GPU ms/s" } else { "CPU ms/s" }.to_owned(),
                                        short(v),
                                    ),
                                    (
                                        "cell".to_owned(),
                                        format!(
                                            "{} · {} · {} Hz",
                                            c.key.label(),
                                            c.key.size(),
                                            c.key.hz
                                        ),
                                    ),
                                ],
                            })
                        })
                        .collect()
                })
                .collect(),
            diverging: false,
            ends: ("cheaper".to_owned(), "costlier".to_owned()),
        }
    });
    view! { <Heatmap model=model/> }
}
