// The scenario and the data of the spec (`apps/shared/dashboard.rs`), which the Rust side of the
// app gives through its `spec` command (`src/payload.rs`). The module asks for them once, before
// the page renders.
import { invoke } from "@tauri-apps/api/core";

/** What the content shows; the active nav item picks it. */
export type Page = "overview" | "messages" | "empty";

export interface NavItem {
  label: string;
  page: Page;
  /** The unread count of the item with the badge. */
  badge: number | null;
}

/** One metric card. */
export interface Kpi {
  label: string;
  delta: string;
  up: boolean;
  /** The sparkline bar heights in px, oldest first. */
  spark: number[];
  /** The startup value, then the values of the live ticks 1 to 97, which repeat from there. */
  values: string[];
}

/** The bar heights in px of a chart series and of its previous period. */
export interface Series {
  bars: number[];
  previous: number[];
}

/** A status of an order: the text and the colors of its pill. */
export interface Status {
  label: string;
  background: string;
  color: string;
}

/** One order of the table. */
export interface Order {
  number: number;
  /** The order number as `#10200`. */
  numberText: string;
  customer: string;
  initials: string;
  avatar: string;
  /** The index of the status in `statuses`, which is its checkbox row. */
  status: number;
  /** The progress in percent. */
  progress: number;
  cents: number;
  /** The amount as `$1,234.56`. */
  amount: string;
}

/** One message of the feed. */
export interface Message {
  author: string;
  initials: string;
  avatar: string;
  time: string;
  body: string;
}

/** The tables that the orders of the list are made from. */
export interface Names {
  first: string[];
  last: string[];
  avatars: string[];
  /** The statuses of orders 0 to 3, which repeat from there. */
  statuses: number[];
}

export interface Spec {
  scenario: "app" | "list";
  /** The number of orders and messages of the app, or of rows of the list. */
  count: number;
  texts: {
    appName: string;
    userName: string;
    userRole: string;
    userInitials: string;
    searchPlaceholder: string;
    exportLabel: string;
    chartTitle: string;
    filtersTitle: string;
    compareLabel: string;
    resetLabel: string;
    applyLabel: string;
    liveLabel: string;
    emptyLabel: string;
  };
  navItems: NavItem[];
  kpis: Kpi[];
  liveIntervalMs: number;
  segments: string[];
  series: Series[];
  /** The grid lines of the chart in px from the top of the plot. */
  gridLines: number[];
  /** The statuses in the order of the filter checkboxes. */
  statuses: Status[];
  menuItems: string[];
  columns: string[];
  /** The column whose title sorts the orders. */
  sortColumn: number;
  /** The orders of the app scenario; the list makes its own (`order.ts`). */
  orders: Order[];
  messages: Message[];
  names: Names;
}

export const spec = await invoke<Spec>("spec");
