//! The chart card: the title, the segmented control that picks the series, and a bar chart of
//! plain boxes over four grid lines. With compare on, each slot also shows the previous period.

use dioxus::prelude::*;

use super::spec;

/// The grid lines from the top of the plot.
const GRID_TOPS: [f32; 4] = {
    let mut tops = spec::GRID_LINES;
    let mut i = 0;
    while i < tops.len() {
        tops[i] -= spec::PLOT_TOP;
        i += 1;
    }
    tops
};

/// The revenue chart of the series `series`.
#[component]
pub fn Chart(series: Signal<usize>, compare: ReadSignal<bool>) -> Element {
    let shown = series();
    let compare = compare();
    rsx! {
        div { class: "card chart",
            div { class: "chart__head",
                span { class: "card__title", {spec::CHART_TITLE} }
                Segments { series }
            }
            div { class: "plot",
                for top in GRID_TOPS {
                    div { class: "grid", top: "{top}px" }
                }
                div { class: "bars",
                    for i in 0..spec::CHART_BARS {
                        div {
                            key: "{i}",
                            class: "slot",
                            class: if compare { "slot--compare" },
                            if compare {
                                div {
                                    class: "bar bar--previous",
                                    height: "{spec::previous_height(shown, i)}px",
                                }
                            }
                            div { class: "bar", height: "{spec::bar_height(shown, i)}px" }
                        }
                    }
                }
            }
        }
    }
}

/// The segmented control: one button per series, the selected one raised.
#[component]
fn Segments(series: Signal<usize>) -> Element {
    let selected = series();
    rsx! {
        div { class: "segments", role: "tablist",
            for (index , label) in spec::SEGMENTS.into_iter().enumerate() {
                button {
                    key: "{index}",
                    class: "segment",
                    class: if index == selected { "selected" },
                    role: "tab",
                    aria_selected: index == selected,
                    onclick: move |_| series.set(index),
                    {label}
                }
            }
        }
    }
}
