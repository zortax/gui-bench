//! The size of the release binary of every shown framework, from the builds of the run.

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
        let rows = Selection::new(&overview, &prefs, APP)
            .frameworks()
            .iter()
            .map(|fw| {
                let mut row = DotRow::of_framework(format!("binary/{fw}"), fw, fw.clone());
                if let Some(bytes) = overview.provenance.binary_bytes(fw) {
                    let mib = bytes as f64 * factor;
                    row.value = Some(mib);
                    row.text = short(mib);
                }
                row
            })
            .collect();
        DotModel {
            rows,
            x: Axis::value("MiB, release build with line tables", "MiB"),
            rules: Vec::new(),
            hide_labels: false,
            factors: false,
        }
    });
    view! {
        <ChartCard title="Binary" note="The size of the release binary of each build, as the run recorded it">
            <DotPlot model=model/>
        </ChartCard>
    }
}
