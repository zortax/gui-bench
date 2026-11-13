// The app scenario: a sidebar with the nav, a top bar with the search, the live switch and the
// export menu, and a scroll container with the page of the active nav item: the overview (the
// metric cards, the chart and filter cards, and a table of orders), the messages feed, or an
// empty page. The root owns the state and hands each part what it reads or writes; the
// stylesheet `dashboard.css` gives every look.
import { Match, Switch, createMemo, createSignal } from "solid-js";
import { createStore } from "solid-js/store";

import { spec } from "../spec";
import { Chart } from "./Chart";
import { Filters } from "./Filters";
import { Kpis } from "./Kpis";
import { Messages } from "./Messages";
import { Orders } from "./Orders";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";
import { createLive } from "./live";

/** The state that the filter card resets. */
export interface View {
  /** The search text, which filters the orders by customer. */
  query: string;
  /** `shown[k]` for status `k` of the filter checkboxes. */
  shown: boolean[];
  compare: boolean;
  /** The index of the selected segment of the chart. */
  series: number;
  /** Whether the orders are sorted by amount. */
  sorted: boolean;
}

/** Returns the view at startup: every status checked, compare off, the first segment, no search. */
function startView(): View {
  return {
    query: "",
    shown: spec.statuses.map(() => true),
    compare: false,
    series: 0,
    sorted: false,
  };
}

/** The dashboard with the orders and the messages of the spec. */
export function Dashboard() {
  const [active, setActive] = createSignal(0);
  const [view, setView] = createStore(startView());
  const live = createLive();
  const page = createMemo(() => spec.navItems[active()].page);
  return (
    <div class="dash">
      <Sidebar active={active()} onSelect={setActive} />
      <div class="main">
        <Topbar
          title={spec.navItems[active()].label}
          query={view.query}
          onQuery={(query) => setView("query", query)}
          live={live}
        />
        <main class="content">
          <Switch>
            <Match when={page() === "overview"}>
              <div class="content__body">
                <Kpis tick={live.tick()} />
                <div class="row2">
                  <Chart
                    series={view.series}
                    compare={view.compare}
                    onSeries={(series) => setView("series", series)}
                  />
                  <Filters view={view} setView={setView} onReset={() => setView(startView())} />
                </div>
                <Orders view={view} onSort={() => setView("sorted", (sorted) => !sorted)} />
              </div>
            </Match>
            <Match when={page() === "messages"}>
              <Messages />
            </Match>
            <Match when={page() === "empty"}>
              <div class="content__body empty">
                <span class="empty__label">{spec.texts.emptyLabel}</span>
              </div>
            </Match>
          </Switch>
        </main>
      </div>
    </div>
  );
}
