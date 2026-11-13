// The list scenario: the header of the orders over a virtual list of `count` order rows
// (TanStack Virtual). Only the rows in and near the window exist; each makes its order from its
// index. A click on the amount title sorts the indices.
import { createVirtualizer } from "@tanstack/solid-virtual";
import { For, createMemo, createSignal } from "solid-js";

import { listOrder, order } from "../order";
import { spec } from "../spec";
import { OrderRow, OrdersHead } from "./Orders";

/** The height of an order row in px, as the stylesheet gives it. */
const ROW_HEIGHT = 52;

/** The orders of a list of `count` rows. */
export function OrderList(props: { count: number }) {
  const [sorted, setSorted] = createSignal(false);
  const indices = createMemo(() => listOrder(props.count, sorted()));
  let rows!: HTMLDivElement;
  const virtualizer = createVirtualizer({
    get count() {
      return props.count;
    },
    getScrollElement: () => rows,
    estimateSize: () => ROW_HEIGHT,
  });
  return (
    <div class="list">
      <OrdersHead sorted={sorted()} onSort={() => setSorted((sorted) => !sorted)} />
      <div class="list__rows" ref={rows}>
        <div class="list__sizer" style={{ height: `${virtualizer.getTotalSize()}px` }}>
          <For each={virtualizer.getVirtualItems()}>
            {(item) => {
              const rowOrder = createMemo(() => order(spec.names, indices()[item.index]));
              return (
                <div class="list__row" style={{ transform: `translateY(${item.start}px)` }}>
                  <OrderRow order={rowOrder()} />
                </div>
              );
            }}
          </For>
        </div>
      </div>
    </div>
  );
}
