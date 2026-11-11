//! The layout of the table view: size, measure and variant pair controls, one dot plot per
//! operation, the speedup heatmap and the churn panels.

use std::sync::Arc;

use explorer_core::api::{MetricRef, RunOverview};
use leptos::prelude::*;

use super::charts::{Churn, OpDots, Speedup};
use crate::state::{RunView, Selection, use_explorer, use_param};
use crate::views::kit::{ViewHead, WithData};
use crate::widgets::{Empty, Segmented};

/// The table operations: the report, the title and what it does.
pub const OPS: [(&str, &str); 8] = [
    ("op-select", "Select a row (click)"),
    ("op-update10", "Update every 10th row"),
    ("op-swap", "Swap two rows"),
    ("op-remove", "Remove a row"),
    ("op-insert", "Insert a row"),
    ("op-replace", "Replace all rows"),
    ("op-append", "Append 1,000 rows"),
    ("op-clear", "Clear or recreate all"),
];

/// Returns the metric of an operation for a measure: latency or CPU per operation.
pub fn op_metric(op: &str, measure: &str) -> MetricRef {
    if measure == "cpu" {
        MetricRef::new(op, "resources", "cpu_ms_per_op")
    } else {
        MetricRef::new(op, "latency", "total_ms")
    }
}

fn metrics() -> Vec<MetricRef> {
    let mut out: Vec<MetricRef> = OPS
        .iter()
        .flat_map(|(op, _)| {
            [
                op_metric(op, "latency"),
                op_metric(op, "cpu"),
                MetricRef::new(op, "latency", "total_p50"),
            ]
        })
        .collect();
    for m in [
        "pss_kept_per_cycle",
        "private_kept_per_cycle",
        "vram_kept_per_cycle",
    ] {
        out.push(MetricRef::new("churn", "memory", m));
    }
    out
}

/// The table view.
#[component]
pub fn TableView() -> impl IntoView {
    let (rows, set_rows) = use_param("rows", "1000");
    let (measure, set_measure) = use_param("measure", "latency");
    let explorer = use_explorer();
    view! {
        <div class="gx-view">
            <ViewHead
                title="Table operations"
                text="A keyed table of N rows; F-keys run each operation and its undo. Input to photon of the first frame that shows the result, or the app CPU time per operation."
            >
                <Segmented
                    options=vec![("latency".to_owned(), "input to photon".to_owned()), ("cpu".to_owned(), "CPU per operation".to_owned())]
                    value=Signal::derive(move || measure.get())
                    on_change=set_measure
                />
                <Segmented
                    options=vec![("1000".to_owned(), "1,000 rows".to_owned()), ("10000".to_owned(), "10,000 rows".to_owned())]
                    value=Signal::derive(move || rows.get())
                    on_change=set_rows
                />
            </ViewHead>
            <RunView view={move |o: Arc<RunOverview>| {
                let prefs = explorer.prefs.snapshot();
                if Selection::new(&o, &prefs, "table").cells.is_empty() {
                    return view! { <Empty message="No table cells shown: the run has none, or the filters hide them."/> }.into_any();
                }
                view! {
                    <WithData overview=o metrics=metrics() skeletons=4 view=move |data| {
                        let d = data.clone();
                        view! {
                            <div class="gx-grid">
                                {OPS.iter().map(|(op, title)| view! {
                                    <OpDots data=d.clone() op=*op title=*title rows=rows measure=measure/>
                                }).collect_view()}
                            </div>
                            <div class="gx-grid gx-grid--wide">
                                <Speedup data=data.clone() rows=rows/>
                                <Churn data=data.clone()/>
                            </div>
                        }
                    }/>
                }
                .into_any()
            }}/>
        </div>
    }
}
