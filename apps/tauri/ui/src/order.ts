// The orders of the list scenario. The list has up to a million rows, so it makes the order of a
// row from its index when the row comes into view, as `order`, `cents` and `list_order` of
// `apps/shared/dashboard.rs` do, from the name tables of the spec.
import type { Names, Order } from "./spec";

/** Returns the amount of order `i` in cents. */
export function cents(i: number): number {
  return ((i * 7919) % 500_000) + 1_000;
}

/** Returns order `i` of the table. */
export function order(names: Names, i: number): Order {
  const first = names.first[(i * 7) % names.first.length];
  const last = names.last[(i * 5 + Math.floor(i / 3)) % names.last.length];
  const amount = cents(i);
  const dollars = Math.floor(amount / 100).toLocaleString("en-US");
  return {
    number: 10200 + i,
    numberText: `#${10200 + i}`,
    customer: `${first} ${last}`,
    initials: first[0] + last[0],
    avatar: names.avatars[(i * 5) % names.avatars.length],
    status: names.statuses[i % names.statuses.length],
    progress: (i * 37) % 101,
    cents: amount,
    amount: `$${dollars}.${String(amount % 100).padStart(2, "0")}`,
  };
}

/**
 * Returns the indices of the orders of the list in the order of its rows: by amount, largest
 * first, when `sorted`, and `0..count` otherwise. Orders with the same amount keep their order.
 */
export function listOrder(count: number, sorted: boolean): Uint32Array {
  const indices = Uint32Array.from({ length: count }, (_, i) => i);
  if (sorted) {
    indices.sort((a, b) => cents(b) - cents(a));
  }
  return indices;
}
