# How the apps are written

gui-bench compares frameworks the way an app author meets them. Every app
follows one rule, and this file lists where an app departs from it.

## The rule

1. **Built-in parts.** An app uses its framework's own scroll container, text
   input, checkbox, switch, button, menu, virtual list, timer and animation.
2. **The documented pattern.** State and views are structured the way the
   framework's documentation and examples structure an app of this kind:
   signals per value, components per part, entities per view, or one `view`
   function, as the framework teaches.
3. **Defaults.** Renderer, present mode, scroll scale, scroll smoothing,
   momentum and scrollbars stay as the framework ships them. No app rewrites
   scroll or key input.
4. **No app-side optimization.** No manual caching, memoization,
   virtualization or skipped work beyond what rule 2 covers. Where a framework
   offers an opt-in cache that its documentation describes, the plain app
   leaves it out and a `tuned` edition uses it, so results show both.
5. **Missing parts are built by hand and flagged.** Where a framework has no
   part of a kind, or its part does not work at the pinned version, the app
   builds the smallest straightforward one. That is what an author of such an
   app would have to do, so it is the idiomatic code for that framework. The
   runner records each such part in `run.json` (`builds.<framework>.flags`),
   and the plots draw its measurements with hollow markers.

## Tuned editions

| Edition | The plain app | The tuned edition |
|---|---|---|
| `gpui-kit-tuned` | embeds every view entity plainly; a notified entity re-renders its ancestors and their other children | embeds the sidebar, the top bar, the metric cards and the orders with `cached`, so GPUI reuses their last frame while they are not notified |
| `iced-tuned` | builds every widget in `view` on every message | wraps the orders card, the message feed and the rows of the list in `lazy`, keyed by the data they show |

Both are one cargo feature (`tuned`) around one helper function per app.

## Parts built by hand

Flagged parts carry their name in `run.json`.

| Framework | Part | Flag | What the app does, and why |
|---|---|---|---|
| egui | switch | `switch` | egui has no switch. The compare and live switches are the `toggle_switch` of the egui demo. |
| egui | spinner, live timer | – | Immediate mode has no animation or timer objects: the angle and the tick come from the input time, with `request_repaint` and `request_repaint_after`, as egui's own `Spinner` does. |
| egui | font weights | – | egui has no weights. Semibold and bold are font families of their own. |
| egui | sorted data | – | The sorted orders live in the app state and are sorted again when the sorting toggles, not on every pass. |
| iced | virtual list | `list` | iced has none. `on_scroll` gives the viewport; `view` builds the visible rows and 4 more on each side between two spaces. |
| iced | menu | `menu` | iced has no menu or popover (`pick_list` is a select as wide as its button). The menu is a container of buttons, pinned in a `stack` over the window. |
| iced | spinner | `spinner` | Widgets have no transform animation. A `canvas` draws the turned square on every frame of `window::frames`. |
| iced | hover color | – | Containers have no hover style. Order rows, nav items and menu items are buttons whose style follows the hover status. |
| iced | switch size | – | A `toggler` is twice as wide as tall: 44x22 in place of 40x22. |
| freya | menu | `menu` | `Menu` has no anchor. The app places it in an absolute box under the button, as freya's `SubMenu` places its own. |
| freya | button shadow | – | The button theme has no shadow. A box around the primary buttons carries it. |
| gpui-kit | spinner | `spinner` | GPUI rotates only `svg` elements. The square is an inline SVG under `with_animation`, as in gpui-component's `Spinner`. |
| vizia | spinner | `spinner` | Animations cannot repeat without end at 0.4.0. A 16 ms timer sets the angle, so the spinner turns at about 62 Hz at any refresh rate. |
| vizia | row rules | – | Vizia has no border per side. The rule under a row is a 1 px gradient background. |
| vizia | line height | – | No `line-height` property. Message lines take the font's height, about 19 px. |
| floem | menu | `menu` | Floem has no styleable menu. The app adds a stack of items with `add_overlay` under the button, as Floem's `Dropdown` does; the declarative `Overlay` view keeps the window from settling at the pinned revision. |
| floem | spinner | `spinner` | A repeating keyframe animation stops after one pass at the pinned revision. The app turns an angle signal from `exec_after_animation_frame` and paints the square on a `canvas`, since vger draws a rotated rounded rect as its bounding box. |
| dioxus | virtual list | `list` | Dioxus has none. `onscroll` gives the viewport; the component renders the visible rows and 8 more on each side between two spacers. |
| dioxus | switch, menu | – | Written as on the web: a `button role=switch`, and a conditionally rendered absolute box for the menu. |
| dioxus | placeholder | – | Blitz does not paint `placeholder`. A span over the input shows it. |
| zgui | – | – | None. |

## Known defects that the apps do not work around

- **Dioxus Native, vello-hybrid:** a CSS animation starts only after an input
  asks for a redraw. The live switch click is such an input.
- **Vizia:** the first frame of the window is empty; the content follows in a
  later frame. The runner takes the startup time to the frame with the
  content (`content_ms`).
