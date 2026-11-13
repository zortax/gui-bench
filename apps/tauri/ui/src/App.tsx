import { Match, Switch } from "solid-js";

import { Dashboard } from "./dashboard/Dashboard";
import { OrderList } from "./dashboard/OrderList";
import { spec } from "./spec";

/** The root component: the scenario of the command line. */
export function App() {
  return (
    <Switch>
      <Match when={spec.scenario === "app"}>
        <Dashboard />
      </Match>
      <Match when={spec.scenario === "list"}>
        <OrderList count={spec.count} />
      </Match>
    </Switch>
  );
}
