//! The size of the binary of every shown framework, from the builds of the run: the size build
//! where the run recorded one, else the release build that the benchmarks ran.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use explorer_core::units::{display, short};
use leptos::prelude::*;

use crate::chart::{Axis, ChartCard, DotModel, DotPlot, DotRow};
use crate::state::{Selection, use_explorer};
use crate::views::kit::APP;

/// A card of the binary sizes.
#[component]
pub fn BinaryCard(overview: Arc<RunOverview>) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let factor = display("bytes").factor;
        let small = overview.provenance.has_min_builds();
        let rows = Selection::new(&overview, &prefs, APP)
            .frameworks()
            .iter()
            .map(|fw| {
                let mut row = DotRow::of_framework(format!("binary/{fw}"), fw, fw.clone());
                let ran = overview.provenance.binary_bytes(fw);
                let shown = if small {
                    overview.provenance.min_binary_bytes(fw)
                } else {
                    ran
                };
                if let Some(bytes) = shown {
                    let mib = bytes as f64 * factor;
                    row.value = Some(mib);
                    row.text = short(mib);
                }
                if let (true, Some(bytes)) = (small, ran) {
                    row.tip.push((
                        "benchmark build, with line tables".to_owned(),
                        format!("{} MiB", short(bytes as f64 * factor)),
                    ));
                }
                row
            })
            .collect();
        DotModel {
            rows,
            x: Axis::value(
                if small {
                    "MiB, built for size and stripped"
                } else {
                    "MiB, release build with line tables"
                },
                "MiB",
            ),
            rules: Vec::new(),
            hide_labels: false,
            factors: false,
        }
    });
    view! {
        <ChartCard title="Binary" note="The size of each app built for the smallest file, where the run recorded such a build; the benchmarks run a release build with line tables">
            <DotPlot model=model/>
        </ChartCard>
    }
}
