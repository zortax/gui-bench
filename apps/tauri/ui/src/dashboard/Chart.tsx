// The chart card: the title, the segmented control that picks the series, and a bar chart of
// plain boxes over four grid lines. With compare on, each slot also shows the previous period.
import { For, Index, Show } from "solid-js";

import { spec } from "../spec";

/** The revenue chart of the series `series`. */
export function Chart(props: {
  series: number;
  compare: boolean;
  onSeries: (series: number) => void;
}) {
  const shown = () => spec.series[props.series];
  return (
    <div class="card chart">
      <div class="chart__head">
        <span class="card__title">{spec.texts.chartTitle}</span>
        <Segments selected={props.series} onSelect={props.onSeries} />
      </div>
      <div class="plot">
        <For each={spec.gridLines}>
          {(top) => <div class="grid" style={{ top: `${top}px` }} />}
        </For>
        <div class="bars">
          <Index each={shown().bars}>
            {(height, i) => (
              <div class="slot" classList={{ "slot--compare": props.compare }}>
                <Show when={props.compare}>
                  <div class="bar bar--previous" style={{ height: `${shown().previous[i]}px` }} />
                </Show>
                <div class="bar" style={{ height: `${height()}px` }} />
              </div>
            )}
          </Index>
        </div>
      </div>
    </div>
  );
}

/** The segmented control: one button per series, the selected one raised. */
function Segments(props: { selected: number; onSelect: (series: number) => void }) {
  return (
    <div class="segments" role="tablist">
      <For each={spec.segments}>
        {(label, index) => (
          <button
            class="segment"
            classList={{ selected: index() === props.selected }}
            role="tab"
            aria-selected={index() === props.selected}
            onClick={() => props.onSelect(index())}
          >
            {label}
          </button>
        )}
      </For>
    </div>
  );
}
