// The filter card: a checkbox per status, the compare switch, and the Reset and Apply buttons.
import { For } from "solid-js";
import type { SetStoreFunction } from "solid-js/store";

import { spec } from "../spec";
import type { View } from "./Dashboard";
import { Switch } from "./Switch";

/** The filters of the orders and the chart. */
export function Filters(props: {
  view: View;
  setView: SetStoreFunction<View>;
  onReset: () => void;
}) {
  return (
    <div class="card filters">
      <span class="card__title">{spec.texts.filtersTitle}</span>
      <div class="checks">
        <For each={spec.statuses}>
          {(status, row) => (
            <label class="check">
              <input
                class="check__box"
                type="checkbox"
                checked={props.view.shown[row()]}
                onChange={(event) => props.setView("shown", row(), event.currentTarget.checked)}
              />
              <span>{status.label}</span>
            </label>
          )}
        </For>
      </div>
      <div class="compare">
        <span>{spec.texts.compareLabel}</span>
        <Switch
          on={props.view.compare}
          onChange={(on) => props.setView("compare", on)}
          label={spec.texts.compareLabel}
        />
      </div>
      <div class="actions">
        <button class="button button--secondary" onClick={() => props.onReset()}>
          {spec.texts.resetLabel}
        </button>
        <button class="button button--primary">{spec.texts.applyLabel}</button>
      </div>
    </div>
  );
}
