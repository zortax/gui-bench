//! The explore view: any metric of a scenario against any other (or against the size), one
//! point per shown cell, colored by framework, hollow where the app builds the part by hand,
//! optionally connected across sizes.

use std::sync::Arc;

use explorer_core::api::{CellInfo, MetricInfo, MetricRef, RunOverview};
use explorer_core::scale::ScaleKind;
use explorer_core::units::display;
use leptos::prelude::*;

use crate::api::metric_catalog;
use crate::chart::{Axis, ChartCard, Legend, LegendItem, Pt, Series, XyModel, XyPlot, use_card};
use crate::palette;
use crate::state::{RunView, cells_of, use_explorer, use_param};
use crate::views::kit::{Build, Data, ViewHead, WithData, cell_tip};
use crate::widgets::{ChartSkeleton, ErrorCard, Segmented, SelectBox, Toggle};

/// The pseudo metric of the size: the elements of the app, or the rows of the list.
const SIZE: &str = "size";

fn label(m: &MetricRef) -> String {
    format!("{} · {} · {}", m.measurement, m.section, m.metric)
}

/// The explore view.
#[component]
pub fn ExploreView() -> impl IntoView {
    view! {
        <div class="gx-view">
            <ViewHead title="Explore" text="Pick a scenario and any two of its metrics. Every shown cell is one point; a click opens it."/>
            <RunView view={move |o: Arc<RunOverview>| view! { <Builder overview=o/> }}/>
        </div>
    }
}

/// Loads the metric catalog of the run, then shows the builder.
#[component]
fn Builder(overview: Arc<RunOverview>) -> impl IntoView {
    let run = overview.summary.id.clone();
    let catalog = Resource::new(move || run.clone(), metric_catalog);
    view! {
        <Transition fallback=|| view! { <ChartSkeleton/> }>
            {move || {
                let overview = overview.clone();
                Suspend::new(async move {
                    match catalog.await {
                        Ok(c) => view! { <Body overview=overview catalog=Arc::new(c)/> }.into_any(),
                        Err(e) => view! { <ErrorCard title="The metric catalog does not load" error=e.to_string()/> }.into_any(),
                    }
                })
            }}
        </Transition>
    }
}

#[component]
fn Body(overview: Arc<RunOverview>, catalog: Arc<Vec<MetricInfo>>) -> impl IntoView {
    let (scenario, set_scenario) = use_param("sc", "app");
    let (x, set_x) = use_param("x", SIZE);
    let (y, set_y) = use_param("y", "hover-row.latency.response_ms");
    let (xs, set_xs) = use_param("xs", "log");
    let (lines, set_lines) = use_param("lines", "1");
    let mut scenarios: Vec<String> = Vec::new();
    for c in &overview.cells {
        if !scenarios.contains(&c.key.scenario) {
            scenarios.push(c.key.scenario.clone());
        }
    }
    let options = Memo::new(move |_| {
        let sc = scenario.get();
        let mut out: Vec<(String, String)> = catalog
            .iter()
            .filter(|m| m.scenarios.contains(&sc))
            .map(|m| (m.metric.to_string(), label(&m.metric)))
            .collect();
        out.sort_by(|a, b| a.1.cmp(&b.1));
        out
    });
    let x_options = Signal::derive(move || {
        let mut o = vec![(SIZE.to_owned(), "size (elements or rows)".to_owned())];
        o.extend(options.get());
        o
    });
    let metrics = Signal::derive(move || {
        [x.get(), y.get()]
            .iter()
            .filter_map(|m| MetricRef::parse(m))
            .collect::<Vec<_>>()
    });
    let sc_options: Vec<(String, String)> = scenarios.into_iter().map(|s| (s.clone(), s)).collect();
    view! {
        <div class="gx-view__controls gx-explore__controls">
            <SelectBox label="scenario" options=sc_options value=Signal::derive(move || scenario.get()) on_change=set_scenario/>
            <SelectBox label="x" options=x_options value=Signal::derive(move || x.get()) on_change=set_x/>
            <Segmented
                options=vec![("lin".to_owned(), "x lin".to_owned()), ("log".to_owned(), "x log".to_owned())]
                value=Signal::derive(move || xs.get())
                on_change=set_xs
            />
            <SelectBox label="y" options=Signal::derive(move || options.get()) value=Signal::derive(move || y.get()) on_change=set_y/>
            <Toggle
                label="connect sizes"
                value=Signal::derive(move || lines.get() == "1")
                on_change=Callback::new(move |on: bool| set_lines.run(if on { "1" } else { "0" }.to_owned()))
            />
        </div>
        <WithData overview=overview metrics=metrics skeletons=1 view=move |data| {
            view! {
                <ChartCard title="Scatter" wide=true>
                    <Scatter data=data scenario=scenario x=x y=y xs=xs lines=lines/>
                </ChartCard>
            }
        }/>
    }
}

fn label_or(m: &str) -> String {
    MetricRef::parse(m).map_or_else(|| m.to_owned(), |m| label(&m))
}

#[component]
fn Scatter(
    data: Data,
    scenario: Memo<String>,
    x: Memo<String>,
    y: Memo<String>,
    xs: Memo<String>,
    lines: Memo<String>,
) -> impl IntoView {
    let explorer = use_explorer();
    let ctx = use_card();
    Effect::new(move |_| {
        ctx.has_spread.set(true);
        ctx.note.set(format!(
            "{} against {} · one point per cell, median · whiskers per the uncertainty setting",
            label_or(&y.get()),
            label_or(&x.get())
        ));
    });
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let sc = scenario.get();
        let xm = MetricRef::parse(&x.get());
        let ym = MetricRef::parse(&y.get());
        let b = Build::new(&data, &prefs, ctx.uncertainty());
        let x_unit = xm.as_ref().map(|m| data.unit(m)).unwrap_or_default();
        let y_unit = ym.as_ref().map(|m| data.unit(m)).unwrap_or_default();
        let x_axis = Axis::linear(
            xm.as_ref().map_or("size".to_owned(), |m| {
                format!("{} ({})", m.metric, display(&x_unit).words)
            }),
            display(&x_unit).label,
        )
        .with_kind(if xs.get() == "log" {
            ScaleKind::Log
        } else {
            ScaleKind::Linear
        });
        let y_axis = Axis::value(
            ym.as_ref().map_or(String::new(), |m| {
                format!("{} ({})", m.metric, display(&y_unit).words)
            }),
            display(&y_unit).label,
        )
        .zero_based(false);
        let mut model = XyModel::new(x_axis, y_axis);
        model.height = 420.0;
        let Some(ym) = ym else { return model };
        let cells: Vec<&CellInfo> = cells_of(data.overview(), &prefs, &sc);
        let mut keys: Vec<(String, f64)> = Vec::new();
        for c in &cells {
            let k = (c.key.framework.clone(), c.key.hz);
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        for (fw, hz) in keys {
            let mut s = Series::of_framework(format!("{fw}/{hz}"), &fw, format!("{fw} {hz} Hz"));
            s.hollow = cells
                .iter()
                .find(|c| c.key.framework == fw)
                .is_some_and(|c| b.hand_built(&c.key, &ym.measurement));
            s.line = lines.get() == "1";
            s.dash = palette::dash(&fw).to_owned();
            s.radius = 4.0;
            for c in cells
                .iter()
                .filter(|c| c.key.framework == fw && c.key.hz == hz)
            {
                let Some(vy) = b.value(&ym, c) else { continue };
                let (px, xw) = match &xm {
                    Some(m) => match b.value(m, c) {
                        Some(vx) => (vx.y, vx.spread.whisker),
                        None => continue,
                    },
                    None => (c.key.size(), None),
                };
                let mut p = Pt::new(px, vy.y);
                p.whisker = vy.spread.whisker;
                p.band = vy.spread.band;
                p.cell = Some(c.id);
                p.invalid = vy.invalid;
                p.tip = cell_tip(c, &vy.stat, data.samples(&ym));
                if let Some((lo, hi)) = xw {
                    p.tip.push((
                        "x whiskers".to_owned(),
                        format!(
                            "{} – {}",
                            explorer_core::units::short(lo),
                            explorer_core::units::short(hi)
                        ),
                    ));
                }
                s.points.push(p);
            }
            s.points.sort_by(|a, b| a.x.total_cmp(&b.x));
            if !s.points.is_empty() {
                model.series.push(s);
            }
        }
        model
    });
    let legend = Signal::derive(move || {
        let mut items: Vec<LegendItem> = Vec::new();
        for s in model.with(|m| m.series.clone()) {
            if !items.iter().any(|i| i.group == s.group) {
                items.push(LegendItem::framework(&s.group));
            }
        }
        items
    });
    view! {
        <Legend items=legend/>
        <XyPlot model=model/>
    }
}
