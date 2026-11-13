// The four metric cards: a label, a value, the delta pill and a sparkline of plain bars. The
// value follows the live tick.
import { For } from "solid-js";

import { type Kpi, spec } from "../spec";

/** Returns the value text of a metric card at a live tick; tick 0 is the startup value. */
function value(kpi: Kpi, tick: number): string {
  const period = kpi.values.length - 1;
  return kpi.values[tick === 0 ? 0 : 1 + ((tick - 1) % period)];
}

/** The row of metric cards, which share the width. */
export function Kpis(props: { tick: number }) {
  return (
    <div class="kpis">
      <For each={spec.kpis}>
        {(kpi) => (
          <div class="card kpi">
            <div class="kpi__label">{kpi.label}</div>
            <div class="kpi__value">{value(kpi, props.tick)}</div>
            <span class="pill kpi__delta" classList={{ up: kpi.up, down: !kpi.up }}>
              {kpi.delta}
            </span>
            <div class="spark">
              <For each={kpi.spark}>
                {(height) => <div class="spark__bar" style={{ height: `${height}px` }} />}
              </For>
            </div>
          </div>
        )}
      </For>
    </div>
  );
}
