//! The revenue chart card: the title, the segmented control and the bar plot.

use zgui::prelude::*;
use zgui_ui::prelude::*;

use super::{Filters, spec};

/// The chart card. The segments pick `filters.segment`; with `filters.compare` every slot shows
/// the previous period beside the current one.
#[component]
pub fn RevenueChart(filters: Filters) -> impl IntoView {
    let segment = filters.segment;
    let compare = filters.compare;
    // The group holds the names of its pressed items; a click on the pressed one would empty it,
    // which a segmented control declines, so the write keeps the last segment instead.
    let segments = Binding::controlled(
        Signal::derive_local(move || vec![spec::SEGMENTS[segment.get()].to_owned()]),
        move |pressed: Vec<String>| {
            if let Some(index) = pressed
                .first()
                .and_then(|name| spec::SEGMENTS.iter().position(|s| s == name))
            {
                segment.set(index);
            }
        },
    );
    view! {
        column(class = "card chart") {
            row(class = "chart__head") {
                text(class = "card__title") {{spec::CHART_TITLE}}
                ToggleGroup(value = segments, label = "Period", class = "segments") {
                    {Vec::from(spec::SEGMENTS.map(|name| view! {
                        ToggleGroupItem(value = name, class = "segment") {{name}}
                    }.into_view()))}
                }
            }
            row(class = "plot", class:comparing = move || compare.get()) {
                {Vec::from(spec::GRID_LINES.map(|y| view! {
                    box(class = "grid", style:top = format!("{}px", y - spec::PLOT_TOP))
                }.into_view()))}
                {(0..spec::CHART_BARS)
                    .map(|i| view! { Slot(index = i, segment = segment) }.into_view())
                    .collect::<Vec<_>>()}
            }
        }
    }
}

/// Slot `index` of the plot: the previous-period bar, shown only with compare, and the bar of the
/// selected series.
#[component]
fn Slot(index: usize, segment: RwSignal<usize, zgui::reactive::LocalStorage>) -> impl IntoView {
    let px = |height: f32| Some(format!("{height}px"));
    view! {
        box(class = "slot") {
            box(
                class = "bar--previous",
                style:height = move || px(spec::previous_height(segment.get(), index))
            )
            box(
                class = "bar--current",
                style:height = move || px(spec::bar_height(segment.get(), index))
            )
        }
    }
}
