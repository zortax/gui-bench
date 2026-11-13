// The sidebar: the brand, the nav items and the user block. A click on a nav item makes it the
// active one; the stylesheet gives the other items their hover color.
import { For, Show } from "solid-js";

import { spec } from "../spec";

/** The dark sidebar. `active` is the index of the active nav item. */
export function Sidebar(props: { active: number; onSelect: (item: number) => void }) {
  return (
    <nav class="sidebar">
      <div class="brand">
        <div class="brand__logo" />
        <span class="brand__name">{spec.texts.appName}</span>
      </div>
      <div class="nav">
        <For each={spec.navItems}>
          {(item, index) => (
            <button
              class="nav__item"
              classList={{ active: props.active === index() }}
              onClick={() => props.onSelect(index())}
            >
              <span class="nav__icon" />
              <span class="nav__label">{item.label}</span>
              <Show when={item.badge !== null}>
                <span class="badge">{item.badge}</span>
              </Show>
            </button>
          )}
        </For>
      </div>
      <div class="user">
        <div class="user__avatar">{spec.texts.userInitials}</div>
        <div>
          <div class="user__name">{spec.texts.userName}</div>
          <div class="user__role">{spec.texts.userRole}</div>
        </div>
      </div>
    </nav>
  );
}
