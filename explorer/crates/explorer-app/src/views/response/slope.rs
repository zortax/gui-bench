//! What a larger app costs: for every framework and interaction, the growth of the median
//! response time per 1,000 elements, from a straight-line fit over the measured sizes.

use explorer_core::story::{RESPONSES, slope_per_thousand, slope_shade};
use explorer_core::units::{compact, short};
use leptos::prelude::*;

use crate::chart::{ChartCard, HeatCell, HeatModel, Heatmap};
use crate::palette;
use crate::state::{Uncertainty, use_explorer};
use crate::views::kit::{APP, Build, Data, app_series, response, wrap};

/// The card of the slopes.
#[component]
pub fn SlopeCard(data: Data) -> impl IntoView {
    let explorer = use_explorer();
    let model = Signal::derive(move || {
        let prefs = explorer.prefs.snapshot();
        let b = Build::new(&data, &prefs, Uncertainty::Auto);
        let main = data.overview().provenance.main_hz();
        let mut frameworks: Vec<String> = Vec::new();
        for c in b.cells(APP) {
            if !frameworks.contains(&c.key.framework) {
                frameworks.push(c.key.framework.clone());
            }
        }
        let cells = frameworks
            .iter()
            .map(|fw| {
                let series = app_series(&b, fw, main);
                RESPONSES
                    .iter()
                    .map(|step| {
                        let m = response(step.name);
                        let points: Vec<(f64, f64)> = series
                            .iter()
                            .filter(|c| !prefs.valid_only || c.is_valid(step.name))
                            .filter_map(|c| Some((c.key.size(), data.stat(&m, c.id)?.median)))
                            .collect();
                        let slope = slope_per_thousand(&points);
                        let mut tip: Vec<(String, String)> = points
                            .iter()
                            .map(|(x, v)| {
                                (
                                    format!("{} elements", compact(*x)),
                                    format!("{} ms", short(*v)),
                                )
                            })
                            .collect();
                        if let Some(s) = slope {
                            tip.push((
                                "slope".to_owned(),
                                format!("{} ms per 1,000 elements", short(s)),
                            ));
                        }
                        Some(HeatCell {
                            text: slope.map_or("–".to_owned(), |s| slope_shade(s).0),
                            t: slope.map(|s| slope_shade(s).1),
                            cell: series.last().map(|c| c.id),
                            invalid: series
                                .iter()
                                .any(|c| data.stat(&m, c.id).is_some() && !c.is_valid(step.name)),
                            tip,
                        })
                    })
                    .collect()
            })
            .collect();
        HeatModel {
            rows: frameworks
                .iter()
                .map(|f| (f.clone(), palette::ink(f)))
                .collect(),
            cols: RESPONSES.iter().map(|s| wrap(s.title, 14)).collect(),
            cells,
            diverging: false,
            ends: ("near zero".to_owned(), "30 ms and more".to_owned()),
        }
    });
    view! {
        <ChartCard
            title="Cost per 1,000 elements"
            note="Milliseconds of response time added per 1,000 elements in the app: the slope of a line through the medians over the sizes · darker is more · near zero: the work follows the change, not the size of the app"
            wide=true
        >
            <Heatmap model=model/>
        </ChartCard>
    }
}
