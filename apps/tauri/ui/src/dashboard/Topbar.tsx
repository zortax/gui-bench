// The top bar: the title of the page, the search input, the live switch with its spinner, the
// export button with its menu, and the avatar. Typing into the search sets the query that
// filters the orders.
import { For, Show, createSignal } from "solid-js";

import { spec } from "../spec";
import { Switch } from "./Switch";
import type { Live } from "./live";

/** The white bar over the content. `title` is the name of the active nav item. */
export function Topbar(props: {
  title: string;
  query: string;
  onQuery: (query: string) => void;
  live: Live;
}) {
  return (
    <header class="topbar">
      <span class="topbar__title">{props.title}</span>
      <input
        class="search"
        type="text"
        placeholder={spec.texts.searchPlaceholder}
        value={props.query}
        onInput={(event) => props.onQuery(event.currentTarget.value)}
      />
      <div class="live">
        <Switch on={props.live.on()} onChange={props.live.setOn} label={spec.texts.liveLabel} />
      </div>
      <div class="live__spinner">
        <Show when={props.live.on()}>
          <div class="spinner" />
        </Show>
      </div>
      <ExportMenu />
      <div class="topbar__avatar">{spec.texts.userInitials}</div>
    </header>
  );
}

/**
 * The export button and the menu it opens under itself. A click on the button or on an item
 * closes the menu.
 */
function ExportMenu() {
  const [open, setOpen] = createSignal(false);
  return (
    <div class="export">
      <button
        class="button button--primary export__button"
        aria-haspopup="menu"
        aria-expanded={open()}
        onClick={() => setOpen((open) => !open)}
      >
        {spec.texts.exportLabel}
      </button>
      <Show when={open()}>
        <div class="menu" role="menu">
          <For each={spec.menuItems}>
            {(label) => (
              <button class="menu__item" role="menuitem" onClick={() => setOpen(false)}>
                {label}
              </button>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
