# gui-bench

Benchmarks of Rust GUI frameworks under
[fakeland](../fakeland), a headless Wayland compositor that fakes the display
and measures every frame from input to presentation.

The suite asks one question: **how does what the user perceives change as an
app grows?** A small app is fast in every framework. The differences show when
the app holds thousands of elements and the user hovers, types, scrolls,
resizes or switches a page.

Each framework has one app in `apps/<framework>` (binary
`<framework>-bench`) that shows one scenario per run:
`<framework>-bench --scenario app|list --rows N`. Every app draws the same
pixels: the layout, colors, texts and data live in `apps/shared/dashboard.rs`,
the command line in `apps/shared/cli.rs`. Each app is written the way the
framework's documentation teaches, with its built-in widgets, and without
shortcuts around its normal update path; [IDIOMS.md](IDIOMS.md) states the
rule and lists every part an app builds by hand.

## Scenarios

### `app`: a realistic app at growing size

An analytics workbench: a dark sidebar with nav items, a top bar with a search
input, a live switch and an export menu, four metric cards with sparklines, a
bar chart with a segmented control, a filter card with checkboxes, a switch and
buttons, and an orders table with avatars, status pills and progress bars, on
rounded cards with drop shadows. The nav switches the page; the messages page
is a feed of cards with wrapping text.

`--rows N` sets the number of orders and of messages. All of them are real
elements, with no virtualization, so N scales the element tree. Results use
the elements the spec counts (228 plus 11 per order) as the size of the app:

| `--rows` | 30 | 300 | 1,000 | 3,000 |
|---|---|---|---|---|
| elements | 558 | 3,528 | 11,228 | 33,228 |

Every cell runs at 144 Hz and measures, in this order:

| Step | The user | Measured |
|---|---|---|
| `startup` | launches the app three times | time to the first frame with the content |
| `idle` | does nothing for 3 s | memory, VRAM, CPU time, wakeups, commits |
| `hover-nav`, `hover-row` | moves the pointer onto a nav item, an order row | response time |
| `click-compare` | turns the compare switch on (the chart gains bars) | response time |
| `click-filter` | unchecks a status (a quarter of the orders leave) | response time |
| `click-sort` | clicks the amount title (the orders reorder) | response time |
| `click-menu` | opens the export menu over the content | response time |
| `key-search` | types a letter into the search (every order leaves) | response time |
| `scroll`, `scroll-wheel`, `scroll-fling` | scrolls the content: a touchpad glide at 800 px/s, a mouse wheel at 13 notches a second, a fling from 8,000 px/s | frame pacing, missed refreshes, how the content follows the input. The wheel moves the content in notches, so it has no missed-refresh count |
| `resize` | resizes the window from 640x480 to 1200x800 in 1 s | time from the new size to its frame, frames of an outdated size |
| `live` | turns the live switch on: four values tick at 10 Hz and a spinner turns | frame pacing, CPU and GPU time per second |
| `page-messages`, `page-overview` | clicks a nav item (the page is replaced: N cards mount, N orders unmount, and back) | response time |
| `key-type` | types a letter into the search beside the N messages (nothing else changes) | response time |
| `resize-messages` | resizes the window over the wrapping text | as `resize` |

**Response time** is the time from the input to the commit of the frame that
shows the response, plus the time until the GPU finished it. It leaves out the
wait for the next refresh, so it is not rounded to refreshes: it shows how the
cost grows long before a framework misses a frame. The reports also hold the
time to the photon at 144 Hz.

Two rules keep a run short:

- **Size ladder.** The sizes of a framework run smallest first. Once its
  median hover response passes 250 ms, its larger sizes do not run. A cell in
  which any response takes over a second, or never comes, also skips its
  remaining steps. Results show those sizes as over budget.
- **60 Hz only where it matters.** A cell that misses more than 1 % of the
  refreshes while it scrolls or animates runs those steps again at 60 Hz
  (`app/<framework>/60hz/<rows>/`), which shows whether the framework holds
  the slower rate.

### `list`: a long list

The order rows of the app in the framework's virtual list, with 10,000 and
1,000,000 rows of data, of which only the visible ones exist. It measures
startup, idle, `hover-row`, `click-sort` (the data reorders under the rows) and
the three scrolls. The fling brings many new rows into view per frame.

Every cell also takes a **layout check**: a screenshot (`screenshot.png`) and
the colors at fixed points; a cell whose app draws another layout stops there.
The `app` cells also keep `screenshot-menu.png`, `screenshot-live.png` and
`screenshot-messages.png`.

### Editions

An edition is another build of a framework's app:

| Edition | Differs from its framework in |
|---|---|
| `gpui-kit-tuned` | the app embeds its views with `cached`, GPUI's opt-in view cache |
| `iced-tuned` | the app wraps its rows and cards in `lazy`, iced's opt-in widget cache |
| `gpui-kit-fast` | GPUI Fast 0.1.14 as core, platform and renderer |
| `freya-056a5b4` | freya at git `056a5b4` (0.6.0-alpha.1) |
| `dioxus-skia` | dioxus-native's Skia renderer |

The plain `gpui-kit` and `iced` apps use no opt-in caching: they show what the
framework does when the author writes the direct code, and the tuned editions
what the opt-in buys. The last three run only by name or with `just full`.

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

## Frameworks

| Framework | App | Source | Renderer |
|---|---|---|---|
| zgui | `apps/zgui` | git branch `feat/fakeland-marks` of zgui, feature `fakeland-marks` | wgpu Vulkan |
| gpui-kit | `apps/gpui-kit` | `gpui-kit` 0.9.1 (GPUI with gpui-base and gpui-component) | wgpu Vulkan |
| gpui-kit-tuned | `apps/gpui-kit` (feature `tuned`, built into `target/tuned`) | the gpui-kit app with cached views | wgpu Vulkan |
| gpui-kit-fast | `apps/gpui-kit-fast` | the gpui-kit app with the `gpui-fast` feature: GPUI Fast 0.1.14 as core, platform and renderer | wgpu Vulkan |
| freya | `apps/freya` | `freya` 0.5.1 | Skia on Vulkan |
| freya-056a5b4 | `apps/freya-056a5b4` | the freya app on git `marc2332/freya` at `056a5b4` (0.6.0-alpha.1, 2026-11-07) | Skia on Vulkan |
| vizia | `apps/vizia` | `vizia` 0.4.0 | Skia on OpenGL (EGL) |
| floem | `apps/floem` | git `lapce/floem` at `1351ffb` (main, 2026-09-23); the 0.2.0 release dates from 2024 | vger on wgpu Vulkan |
| iced | `apps/iced` | `iced` 0.14.0 | wgpu Vulkan |
| iced-tuned | `apps/iced` (feature `tuned`, built into `target/tuned`) | the iced app with lazy widgets | wgpu Vulkan |
| egui | `apps/egui` | `eframe` 0.37.0 | wgpu Vulkan |
| dioxus | `apps/dioxus` | `dioxus` and `dioxus-native` 0.8.0-alpha.3 (Dioxus Native on Blitz 0.3.0-beta.4) | vello-hybrid on wgpu Vulkan |
| dioxus-skia | `apps/dioxus` (feature `skia`, built into `target/skia`) | the dioxus app with dioxus-native's Skia renderer | Skia |

Crates come from crates.io unless the table says otherwise. All apps share
the command line in `apps/shared/cli.rs` and the spec in
`apps/shared/dashboard.rs`.

Each app is its own cargo workspace, so the GUI stacks never share a
dependency resolve. `gpui-kit-tuned`, `iced-tuned` and `dioxus-skia` are
feature builds of their app into `target/tuned` and `target/skia`
(`just build-editions`).
`gpui-kit-fast` and `freya-056a5b4` pin dependency versions that cannot share
a lock file with their framework (GPUI Fast pins `taffy` and others exactly;
the freya revision links another skia-bindings), so each is its own small
workspace whose `src` links to the framework's sources. Plots give an edition
its framework's hue one step deeper and a dashed line.

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
just build                     # the apps, the editions and the runner, release
just build-app iced            # one app
just run                       # the default matrix: one to two hours
just quick                     # two sizes of the app, one of the list, fewer repeats
just full                      # every edition, five iterations, every size
just scenario list             # one scenario for every framework
just plots                     # plots of results/latest
```

`just run` takes runner options, for example
`just run --scenarios app --frameworks zgui,egui --rows 300,3000`.

The runner stops on a host with the `powersave` CPU governor, which makes the
compositor wake late and the reports invalid; set `performance`, or pass
`--allow-noisy-host`, which also turns the repeats below off. A measurement
that the host disturbs runs again, at most twice; a report that is still
invalid says so (`validity`), and the plots mark it.

Results go to `results/<run>/`: `run.json` (format 3) indexes the cells, the
binary size of every build, the parts each app builds by hand, and the version
of every framework with its git revision where it is no release
(`framework_versions`, read from the app's `Cargo.lock`). Every cell
directory `<scenario>/<framework>/<hz>hz/<rows>/` holds `cell.json` (arguments,
done, failed and skipped measurements, whether it missed refreshes, or why it
did not run), `app.log`, the screenshots, `startup.json` (one entry per
launch) and one fakeland bench report per step (zgui also
`phases-<name>.json`). `results/latest` links to the newest run; `--keep-latest`
leaves it. `just run --resume --out results/<run>` continues a run: cells whose
`cell.json` has no errors keep their results, the others run again. With other
`--scenarios`, `--frameworks` or `--rows` it grows a run; `history` in
`run.json` lists every invocation.

## Plots

`plots/` holds Python scripts (matplotlib only). They set their text in
Libertinus Sans, which ships in `plots/fonts/` with its license (SIL Open
Font License). Colors follow the Oldworld scheme: its light theme for
the page and text, its pastel hues for the frameworks, each with a darker
ink of the same hue for labels. `plots/all.py <run>` writes seven figures
into `<run>/plots/` and a tidy `<run>/summary.csv`. Each figure answers one
question:

| Figure | Question |
|---|---|
| `capacity.png` | How large can the app grow before an interaction misses a 144 Hz or a 60 Hz frame? One number per framework and interaction: the elements up to which the 95th percentile of the response time stays within the frame, and up to which scrolling and animation miss at most 1 % of the refreshes. |
| `response.png` | How does the response time of each interaction grow with the app? A flat line is work that follows the change; a rising line is work that follows the size of the app. |
| `slope.png` | What does each 1,000 elements more cost, in milliseconds of response time? |
| `smoothness.png` | How do the slowest frames and the missed refreshes of scrolling, animation and resizing grow with the app? |
| `timelines.png` | What does a fling look like, frame by frame, at the largest size every framework ran? |
| `baseline.png` | What does a small app cost: startup, memory, work at rest, hover response, binary size? |
| `list.png` | How does the virtual list respond, scroll and use memory at 10,000 and 1,000,000 rows? |

Hollow markers mark a part that the app builds by hand because its framework
has none. Each script also runs on its own:
`python3 plots/response.py results/latest`.

## Method notes

- The output is 1280x800 at scale 1 and 144 Hz. Times are compositor
  timestamps; presentation is the modeled refresh of the fake display.
- **Probes.** A hover, click or key probe sends its input at a planned phase
  of the refresh and finds the response in the pixels of a region: the first
  commit that changes more than 0.1 % of them. Between repeats a second input
  brings the state back (the pointer leaves, the same control is clicked
  again, BackSpace, or the other nav item). A hover probe takes 3 × 12
  samples, the others 3 × 8.
- **Response time and the refresh.** An idle app draws when the input
  arrives and does not wait for a frame callback, so the time from input to
  commit does not depend on the phase of the input (checked for zgui,
  gpui-kit, egui and dioxus: |r| ≤ 0.27). That makes it a continuous measure
  at any refresh rate.
- **Scroll.** The glide and the fling use the touchpad (finger) source at
  four events per refresh; the glide has no kinetic tail. The wheel sends
  whole notches. Every app scrolls with its framework's scroll container as
  it ships, with the framework's own scale, smoothing and momentum, so the
  scroll analyzer first measures each app's content gain (content pixels per
  scroll pixel) and judges steps and stalls against it. Motion estimation
  reads the content above the row under the pointer, since some apps update
  the hover a frame late. A size too short for a scroll skips it (30 orders
  skip the glide and the fling).
- **Missed refreshes** are refreshes without a new frame while one was due,
  as a share of the refreshes of the measurement. A wheel moves the content
  in notches with rests between them, so it has no such count.
- **Startup** runs in its own session per launch. Its time ends with the
  frame after which the window shows the layout of the spec (`content_ms`);
  `first_present_ms` is the first frame of any content, which is empty in
  some frameworks.
- **Page switches** replace the page: the old one is dropped and the new one
  built, in the way each framework switches a view. Filter state lives
  outside the page and survives.
- **The live switch** starts a 100 ms timer of the framework and the
  spinner, with the framework's animation where it has one
  ([IDIOMS.md](IDIOMS.md) lists the exceptions).
- **Hover** changes the color at once, with no transition, in every app.
- Sessions run no accessibility bus by default, as on a desktop without an
  assistive technology: frameworks that publish an AccessKit tree keep it
  dormant. `--a11y` runs the bus, so every app publishes its tree; that
  costs some frameworks much more per frame than others (freya and egui
  rebuild a tree of every row each frame). `run.json` records the setting.
- A report is invalid when the compositor woke late or the host was busy;
  each report carries its validity and the host environment. Runs of several
  matrices at once share the GPU and can exhaust its memory during the resize
  steps, so run one at a time.
- Without app marks, fakeland assigns each input to the next commit by time,
  so per-frame expected scroll steps jitter by an event for gpui-kit. The
  totals and the pixel-based measures are unaffected. zgui sends app marks;
  its cells also hold `<step>-marks` probes and `phases-<step>.json`.
- floem and vizia set no Wayland app id; the runner does not need one.
- At 1,000,000 rows the hand-built lists of iced and dioxus place rows with
  32-bit floats, so rows far down the list can sit a few pixels off.
