// The orders card: the column header and one row per order that passes the status filter and
// the search, sorted by amount after a click on the amount title. Every row is a real element;
// `<For>` keys the rows by their order, so a filter change or a sort keeps the rows whose order
// stays.
import { For, Show, createMemo } from "solid-js";

import { type Order, spec } from "../spec";
import type { View } from "./Dashboard";

/** The class of each column, in the order of the column titles. */
const COLUMN_CLASSES = ["col-order", "col-customer", "col-status", "col-progress", "col-amount"];

/** The table of the orders, filtered and sorted by `view`. */
export function Orders(props: { view: View; onSort: () => void }) {
  const passing = createMemo(() => {
    const query = props.view.query.toLowerCase();
    const shown = props.view.shown;
    return spec.orders.filter(
      (order) => shown[order.status] && order.customer.toLowerCase().includes(query),
    );
  });
  const arranged = createMemo(() =>
    props.view.sorted ? passing().toSorted((a, b) => b.cents - a.cents) : passing(),
  );
  return (
    <div class="card orders">
      <OrdersHead sorted={props.view.sorted} onSort={props.onSort} />
      <For each={arranged()}>{(order) => <OrderRow order={order} />}</For>
    </div>
  );
}

/** The column titles. A click on the amount title asks for the other sorting. */
export function OrdersHead(props: { sorted: boolean; onSort: () => void }) {
  return (
    <div class="orders__head">
      <For each={spec.columns}>
        {(title, index) => (
          <Show
            when={index() === spec.sortColumn}
            fallback={<span class={COLUMN_CLASSES[index()]}>{title}</span>}
          >
            <button class="sort" classList={{ sorted: props.sorted }} onClick={() => props.onSort()}>
              {title}
            </button>
          </Show>
        )}
      </For>
    </div>
  );
}

/**
 * One order: its number, the customer with an avatar, the status pill, the progress bar and the
 * amount. The stylesheet gives it the hover color.
 */
export function OrderRow(props: { order: Order }) {
  const status = () => spec.statuses[props.order.status];
  return (
    <div class="order">
      <span class="col-order order__number">{props.order.numberText}</span>
      <div class="col-customer">
        <span class="avatar" style={{ "background-color": props.order.avatar }}>
          {props.order.initials}
        </span>
        <span>{props.order.customer}</span>
      </div>
      <div class="col-status">
        <span
          class="pill status"
          style={{ "background-color": status().background, color: status().color }}
        >
          {status().label}
        </span>
      </div>
      <div class="col-progress">
        <div class="progress">
          <div class="progress__fill" style={{ width: `${props.order.progress}%` }} />
        </div>
      </div>
      <span class="col-amount order__amount">{props.order.amount}</span>
    </div>
  );
}
