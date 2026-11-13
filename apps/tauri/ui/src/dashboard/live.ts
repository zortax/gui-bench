// The live updates: while the switch of the top bar is on, a timer advances the tick that the
// metric cards show.
import { type Accessor, type Setter, createEffect, createSignal, onCleanup } from "solid-js";

import { spec } from "../spec";

/** The live state: whether it is on, and the tick of the metric values. */
export interface Live {
  on: Accessor<boolean>;
  setOn: Setter<boolean>;
  /** The tick the metric cards show; 0 while live is off. */
  tick: Accessor<number>;
}

/**
 * Returns the live state, off at tick 0. While it is on, an interval counts the tick up; turning
 * it off clears the interval and sets the tick back to 0.
 */
export function createLive(): Live {
  const [on, setOn] = createSignal(false);
  const [tick, setTick] = createSignal(0);
  createEffect(() => {
    if (!on()) return;
    const timer = setInterval(() => setTick((tick) => tick + 1), spec.liveIntervalMs);
    onCleanup(() => {
      clearInterval(timer);
      setTick(0);
    });
  });
  return { on, setOn, tick };
}
