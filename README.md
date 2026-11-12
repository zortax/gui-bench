# gui-bench

Benchmarks of Rust GUI frameworks under
[fakeland](../fakeland), a headless Wayland compositor that fakes the display
and measures every frame from input to presentation.

Each framework has one app in `apps/<framework>` (binary
`<framework>-bench`) that shows one scenario per run, picked on the command
line: `<framework>-bench --scenario NAME --rows N [--variant V]`. Every app
draws the same pixels: the layout, colors, data and table operations live in
`apps/shared/spec.rs`, the command line in `apps/shared/cli.rs`. Each scenario
is written the way the framework's documentation teaches, with its built-in
widgets, keyed lists, timers and animation API, and without shortcuts around
its normal update path.

## Scenarios

| Scenario | The app shows | Measured | Sizes, rates |
|---|---|---|---|
| `rows` | one scroll container of N real rows (no virtualization) | startup, hover latency, scroll smoothness | 10 to 10,000 rows; 60, 144 Hz |
| `table` | a header and a keyed table; F1 to F8 run operations, a click selects a row | input to photon and app CPU time of select, update every 10th row, swap, remove, insert, replace all, append 1,000, clear; memory kept per replace-all cycle (churn) | 1,000, 10,000; 60 Hz |
| `ticker` | the table, and a timer that updates 1 or 100 rows (spread over the list) and the header every 16 ms | CPU and GPU time per second, delivered frames, damaged area | 1,000, 10,000 rows × 1, 100 updates; 60, 144 Hz |
| `spinner` | a 24 px square turning once a second in the header, above static rows | CPU and GPU time per second, delivered frames, damaged area | 0, 10,000 rows; 60, 144 Hz |
| `typing` | a focused text input above static rows | key to photon at 120 wpm | 1,000, 10,000; 60 Hz |
| `search` | the same input, the rows filter on every key | key to filtered list (`z` hides all rows, BackSpace brings them back), typing latency | 1,000, 10,000; 60 Hz |
| `virtual` | the rows of `rows` in the framework's virtualized list | startup, hover latency, scroll smoothness | 100,000, 1,000,000; 60, 144 Hz |
| `resize` | the `rows` app | a resize storm (640x480 to 1200x800 in 1 s), a scale switch 1 to 2 to 1, memory at scale 2 | 100, 10,000; 60 Hz |
| `dashboard` | an analytics dashboard (`apps/shared/dashboard.rs`): a dark sidebar with nav items, a top bar with a search input and buttons, four metric cards with sparklines, a bar chart with a segmented control, a filter card with checkboxes, a switch and buttons, and an orders table with avatars, status pills and progress bars; rounded cards with drop shadows; the content scrolls | hover latency of a nav item and of an order row, click latency of the compare switch (chart redraw) and of a status filter (table update), scroll smoothness over the orders | 50, 1,000 orders; 60, 144 Hz |
| `dashboard-resize` | the dashboard | the measurements of `resize` | 50, 1,000 orders; 60 Hz |

Every cell also measures:

- **Startup:** spawn to the first presented frame.
- **Layout check:** a screenshot (`screenshot.png`) and the colors at fixed
  points; a cell whose app draws another layout stops there.
- **Idle:** three seconds with no input, the middle second suspended
  (xdg `suspended` state): the memory and VRAM footprint, idle CPU time,
  wakeups, commits, and commits while suspended.
- **Resources and frame cost** on every measurement (see Metrics).

### Variants: fine vs coarse

The `table` and `ticker` scenarios run in every variant the framework has:

| Framework | `fine` | `coarse` | `shared` |
|---|---|---|---|
| zgui | `KeyedFields` with one lens per field, an order signal, keyed `for`, `Selector` for the selection | one `RwSignal<Vec<Row>>`; the list is one reactive hole that rebuilds every row | as fine, every row compares one selection signal |
| floem | `dyn_stack` keyed by id, `RwSignal` per row field | the whole list rebuilt from one `RwSignal<Vec<Row>>` | as fine, every row style reads one selection signal |
| vizia | `List` over per-row signals | a binding over one `Signal<Vec<Row>>` that rebuilds every row | as fine, every row maps one selection signal |
| freya | `Row` components with `PartialEq` props, keyed, per-row `State` | rows built inline in the root component | as fine, every row reads one selection `State` |
| gpui-kit | an `Entity` per row embedded with `.cached()`, notified alone | one view over a `Vec`, `cx.notify()` on any change (GPUI's default) | – |
| iced | `keyed_column` with `lazy((id, version, selected))` per row | a plain `column` rebuilt in `view` (iced's default) | – |
| egui | – (`default`: immediate mode builds every row every frame) | | |
| dioxus | keyed `Row` components reading a dioxus-stores `Store<HashMap<id, Item>>` per id, an `is_selected` prop | rows built inline in the root from one `Signal<Vec<Row>>` | as fine, every row reads one selection signal |

`plots/table.py` draws the coarse/fine latency ratio per framework and
operation. Some frameworks redo more than the variant suggests: GPUI marks
the parents of a notified entity dirty, so the table view re-renders (cached
rows are reused); iced runs `view` on every message, with `lazy` skipping
unchanged rows; GPUI's `with_animation` and iced's `window::frames` re-render
the whole page on every spinner frame. These are the frameworks' normal
behavior and part of what the benchmark shows.

## Metrics

fakeland samples every launched process tree every 50 ms
(`SamplingConfig`), and every measurement adds two analyzers:

- **`resources`**: app CPU time (`cpu_ms`, `cpu_ms_per_s`, exact from
  schedstat), wakeups (`wakeups_per_s`, context switches of every thread),
  memory (`rss_max`, `pss_max`, `private_max`), per-process VRAM and GPU busy
  time (`gpu_proc_mem_max`, `gpu_proc_busy_percent`; DRM fdinfo on amdgpu,
  i915, xe and others, NVML on NVIDIA, loaded at run time), and device power
  (`cpu_power_w` from RAPL when readable, `gpu_power_w`). Power is
  device-wide and includes the rest of the host, so it is context only; CPU
  and GPU time per second are the power proxy.
- **`frame_cost`**: frames, CPU ms per frame, GPU time per frame from commit
  to buffer fence (`gpu_fence_ms_p50`, `gpu_fence_ms_per_s`, an upper bound
  on GPU work that works on every driver), and the damaged fraction of the
  window per frame (`damage_fraction_mean`, `full_damage_frames`).
  The GPU time needs a fence on the buffer (explicit or implicit sync); a
  buffer without one counts as ready at commit and gives no GPU time. On
  this host every app damages its whole surface on every frame, since the
  Vulkan and GL presents of the drivers report full damage, so the damaged
  area tells the frameworks apart only where one does partial presents.

CPU time per table operation is the `cpu_ms` of an iteration over its
operations: every probe repeat runs the operation and its undo.

## Frameworks

| Framework | App | Source | Renderer |
|---|---|---|---|
| zgui | `apps/zgui` | git branch `feat/fakeland-marks` of zgui, feature `fakeland-marks` | wgpu Vulkan |
| gpui-kit | `apps/gpui-kit` | `gpui-kit` 0.9.1 (GPUI with gpui-base and gpui-component) | wgpu Vulkan |
| gpui-kit-fast | `apps/gpui-kit-fast` | the gpui-kit app with the `gpui-fast` feature: GPUI Fast 0.1.14 as core, platform and renderer | wgpu Vulkan |
| freya | `apps/freya` | `freya` 0.5.1 | Skia on Vulkan |
| freya-056a5b4 | `apps/freya-056a5b4` | the freya app on git `marc2332/freya` at `056a5b4` (0.6.0-alpha.1, 2026-11-07) | Skia on Vulkan |
| vizia | `apps/vizia` | `vizia` 0.4.0 | Skia on OpenGL (EGL) |
| floem | `apps/floem` | git `lapce/floem` at `1351ffb` (main, 2026-09-23); the 0.2.0 release dates from 2024 | vger on wgpu Vulkan |
| iced | `apps/iced` | `iced` 0.14.0 | wgpu Vulkan |
| egui | `apps/egui` | `eframe` 0.37.0 | wgpu Vulkan |
| dioxus | `apps/dioxus` | `dioxus` and `dioxus-native` 0.8.0-alpha.3 (Dioxus Native on Blitz 0.3.0-beta.4) | vello-hybrid on wgpu Vulkan |
| dioxus-skia | `apps/dioxus` (feature `skia`, built into `target/skia`) | the dioxus app with dioxus-native's Skia renderer | Skia |

Crates come from crates.io unless the table says otherwise. All apps share
the command line in `apps/shared/cli.rs` and the spec in `apps/shared/spec.rs`.

Each app is its own cargo workspace, so the GUI stacks never share a
dependency resolve. An edition of a framework is another build of the same sources. `dioxus-skia`
is a feature build of `apps/dioxus` into `target/skia` (`just build-editions`).
`gpui-kit-fast` and `freya-056a5b4` pin dependency versions that cannot share
a lock file with their framework (GPUI Fast pins `taffy` and others exactly;
the freya revision links another skia-bindings), so each is its own small
workspace whose `src` links to the framework's sources. Plots give an edition
its framework's hue one step deeper.

The runner (its own workspace in `runner/`) depends on fakeland's `main` branch on GitHub, with the
`nvml` feature for per-process GPU numbers on NVIDIA. The zgui app depends on zgui's
`feat/fakeland-marks` branch. To build against local checkouts beside this repository, put the
patches in a `.cargo/config.toml` of the workspace; git ignores these files:

```toml
# runner/.cargo/config.toml
[patch."https://github.com/zortax/fakeland"]
fakeland = { path = "../../fakeland/crates/fakeland" }
fakeland-bench = { path = "../../fakeland/crates/fakeland-bench" }
fakeland-trace = { path = "../../fakeland/crates/fakeland-trace" }
fakeland-types = { path = "../../fakeland/crates/fakeland-types" }

# apps/zgui/.cargo/config.toml
[patch."https://github.com/zortax/zgui"]
zgui = { path = "../../../zgui-fakeland-marks/crates/zgui" }
zgui-ui = { path = "../../../zgui-fakeland-marks/crates/zgui-ui" }
zgui-ui-tokens = { path = "../../../zgui-fakeland-marks/crates/zgui-ui-tokens" }
```

## Run

```sh
just build                     # the apps and the runner, release
just build-app iced            # one app
just run                       # the full matrix: several hours
just quick                     # the smallest sizes at 60 Hz, fewer repeats
just scenario table            # one scenario for every framework
just plots                     # plots of results/latest
```

`just run` takes runner options, for example
`just run --scenarios table,ticker --frameworks zgui,floem --variants fine,coarse --rows 10000 --hz 60`.
Results go to `results/<run>/`: `run.json` (format 2) indexes the cells, and
every cell directory `<scenario>/<framework>/<variant>/<hz>hz/<size>/` holds
`cell.json` (arguments, done and failed measurements), `app.log`,
`screenshot.png`, `startup.json` and one fakeland bench report per
measurement (`idle.json`, `hover.json`, `scroll.json`, `op-<name>.json`,
`churn.json`, `pacing.json`, `typing.json`, `filter.json`, `resize.json`,
`scale.json`, `hover-nav.json`, `hover-row.json`, `click-compare.json`,
`click-filter.json`; zgui also `phases-<name>.json`). The size is the row count,
with `-u<updates>` for the ticker. `results/latest` links to the newest run. `just run --resume --out results/<run>`
continues a run: cells whose `cell.json` has no errors keep their results, the
others run again. With `--scenarios`, it grows a run by scenarios: the cells of the
other scenarios stay in `run.json`, and its `history` lists every invocation.

## Plots

`plots/` holds Python scripts (matplotlib only). They set their text in
Libertinus Sans, which ships in `plots/fonts/` with its license (SIL Open
Font License). Colors follow the Oldworld scheme: its light theme for
the page and text, its pastel hues for the frameworks, each with a darker
ink of the same hue for labels. `plots/all.py <run>` writes
every plot into `<run>/plots/<scenario>/` and a tidy `<run>/summary.csv`:

- `rows/`, `virtual/`: hover latency (linear and log), hover stages, scroll,
  startup against rows; `rows/zgui_phases.png` from zgui's app marks.
- `table/`: latency and CPU time per operation (one figure per row count),
  the coarse/fine speedup, the memory kept per replace-all cycle.
- `ticker/`, `spinner/`: CPU and GPU time per second, delivered frames,
  damaged area, wakeups, per refresh rate.
- `typing/`: typing latency with a static and a filtering list, and filter
  latency.
- `resize/`, `dashboard-resize/`: configure to present, frames at a stale
  size, memory at scale 2.
- `dashboard/`: one figure per refresh rate: hover and click latencies, the
  scroll glide, startup and the idle footprint, per table size.
- `footprint/`: PSS and VRAM against rows with the slope per row, and the
  idle figure.
- `efficiency.png`: CPU and GPU time per second of every scenario's steady
  phase, one row per framework.

Each script also runs on its own: `python3 plots/table.py results/latest`.

## Method notes

- The output is 1280x800 at scale 1. Rows are 32 px tall and start at the top
  of the window. The hover probe enters row 5 from row 1, at planned phases
  of the refresh, and detects the response by the pixels of row 5.
- Scroll uses the touchpad (finger) source without kinetic scrolling, at four
  events per refresh, so every frame consumes the same scroll. Motion
  estimation reads the rows above the pointer, since the hovered row under
  the pointer does not move with the content. Lists shorter than a glide
  (10 rows) have no scroll results.
- Every app scrolls with its framework's scroll container as it ships: no
  app rewrites scroll input, and each keeps the framework's own scale,
  smoothing and momentum. Frameworks scale touchpad distances differently
  (GPUI by 3, freya by 2 with an animation), so the scroll analyzer first
  measures each app's content gain, the content pixels per scroll pixel,
  from the middle of the glide. It searches each frame step near the input
  step times that gain, and judges step errors and stalls against it. The
  `tracking_ratio` metric reports the measured content distance over the
  input distance, which is the gain. The smoothness score compares each
  refresh's step with the next one, so it is the same at any gain.
- Hover changes the row color at once, with no animation, in every app.
- In `rows`, every row is a real element or widget, built up front. egui is immediate
  mode: it builds and lays out all rows every frame, each the usual way (a
  `Frame` with a horizontal layout of two `Label`s, and a `Separator`).
- The scroll region for motion estimation ends six rows above the pointer.
  Some apps update the hover a frame late, so during a glide the hover color
  trails the pointer into the content above it.
- Frame pacing of a glide counts the frames from the first scroll input to
  the last frame that consumes one. A redraw that only answers the end of the
  gesture stays out.
- floem and vizia set no Wayland app id; the runner does not need one.
- Without app marks, fakeland assigns each input to the next commit by time,
  so per-frame expected scroll steps jitter by an event for gpui-kit. The
  totals and the pixel-based measures are unaffected.
- Sessions run no accessibility bus by default, as on a desktop without an
  assistive technology: frameworks that publish an AccessKit tree keep it
  dormant. `--a11y` runs the bus, so every app publishes its tree; that
  costs some frameworks much more per frame than others (freya and egui
  rebuild a tree of every row each frame), and the frameworks publish trees
  of very different detail (zgui only its controls, freya every text).
  `run.json` records the setting.
- Times are compositor timestamps. A run is invalid when the compositor wakes
  late; each report carries its validity and the host environment.
- The header scenarios have no hover color and no zebra stripes, so a click
  or the pointer never changes pixels that a probe watches, and no row's look
  depends on its position.
- Table operations come from F-keys, which reach a window key listener with
  no focused widget in every framework. Each operation has an undo key, so
  the probe alternates between two states.
- The ticker timer and the spinner use each framework's own timer and
  animation API. vizia has no repeating keyframe animation and egui no
  animation system: vizia steps a rotation signal from a 16 ms timer (so its
  spinner moves at about 62 Hz at any refresh rate), egui paints the rotated
  square and asks for a repaint, as its own `Spinner` does.
- floem at `1351ffb` stops a repeating keyframe animation after its first
  pass, so its spinner advances a rotation signal from
  `exec_after_animation_frame`, floem's per-frame callback, and paints the
  square on a `canvas` (vger draws a rotated rounded rect as its bounding
  box). floem also drops a focus request made while the views are built; the
  search input asks again from a zero timer, as floem's own dropdown does.
- Dioxus Native (Blitz) styles with a stylesheet and turns the spinner with a
  CSS `@keyframes` animation. With vello-hybrid the animation does not start
  until an input asks for a redraw (blitz-shell paints the first frame without
  asking for the next one while animating); the layout check moves the
  pointer, which starts it. Dioxus has no virtualized list; `virtual` renders the visible range
  from `onscroll`. Keys come from dioxus-native's window event hook, since
  Blitz sends key events to the focused node only.
- iced has no virtualized list; its `virtual` scenario builds the visible
  rows from the `on_scroll` viewport, the approach the iced docs suggest.
- The text input is each framework's built-in single-line input, focused at
  startup, so its look and height differ slightly; the caret blinks at the
  framework's default rate.
