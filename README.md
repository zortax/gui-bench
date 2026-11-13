# gui-bench

Benchmarks of Rust GUI frameworks under [fakeland](https://github.com/zortax/fakeland), a headless
Wayland compositor that sends real input and times every frame.

Small apps are fast everywhere. This suite measures what happens as an app
grows: how long a hover, a key, a sort or a page switch takes, and whether
scrolling, resizing and animation stay smooth.

## Scenarios

Every framework has one app in `apps/<framework>`
(`<framework>-bench --scenario app|list --rows N`). All apps draw the same
pixels from one spec, `apps/shared/dashboard.rs`.

**`app`** is an analytics workbench: sidebar, top bar with search, live switch
and export menu, metric cards, a bar chart, filters, an orders table, and a
messages page with wrapping text. `--rows` sets the number of orders and
messages. They are real elements, not virtualized, so the element tree grows
with it:

| `--rows` | 30 | 300 | 1,000 | 3,000 |
|---|---|---|---|---|
| elements | 558 | 3,528 | 11,228 | 33,228 |

**`list`** is the same order rows in the framework's virtual list, with 10,000
and 1,000,000 rows.

Each cell runs at 144 Hz and takes these steps:

| Step | Input | Result |
|---|---|---|
| `startup` | three launches | time to the first frame with the content |
| `idle` | none for 3 s | memory, VRAM, CPU time, wakeups |
| `hover-nav`, `hover-row` | pointer onto a nav item, an order row | response time |
| `click-compare`, `click-filter`, `click-sort`, `click-menu` | chart toggle, status filter, sort, export menu | response time |
| `key-search`, `key-type` | a key that hides every order; a key beside the messages | response time |
| `page-messages`, `page-overview` | nav click that replaces the page | response time |
| `scroll`, `scroll-wheel`, `scroll-fling` | touchpad glide, mouse wheel, fling | frame pacing, missed refreshes |
| `resize`, `resize-messages` | 640x480 to 1200x800 in 1 s | time from new size to its frame |
| `live` | four values at 10 Hz and a spinner | frame pacing |

`list` takes `startup`, `idle`, `hover-row`, `click-sort` and the scrolls.

**Response time** runs from the input to the finished frame (commit plus GPU),
without the wait for the next refresh. It is continuous, so it shows growth
long before a frame is missed.

To keep a run short:

- Sizes run smallest first. A framework whose hover takes over 250 ms, or any
  response over a second, skips the rest; results mark those sizes as over
  budget.
- A cell that misses more than 1 % of the refreshes while scrolling or
  animating repeats those steps at 60 Hz.

## Frameworks

| Name | Stack | Renderer |
|---|---|---|
| zgui | zgui, branch `feat/fakeland-marks` | wgpu |
| gpui-kit | gpui-kit 0.9.1 (GPUI, gpui-component) | wgpu |
| freya | freya 0.5.1 | Skia, Vulkan |
| vizia | vizia 0.4.0 | Skia, OpenGL |
| floem | floem at `1351ffb` | vger, wgpu |
| iced | iced 0.14.0 | wgpu |
| egui | eframe 0.37.0 | wgpu |
| dioxus | dioxus-native 0.8.0-alpha.3 (Blitz) | vello-hybrid, wgpu |
| tauri | Tauri 2 with SolidJS, as the web-stack reference | WebKitGTK |

Editions are other builds of an app:

| Edition | Difference |
|---|---|
| gpui-kit-tuned | views embedded with `cached` |
| iced-tuned | rows and cards wrapped in `lazy` |
| gpui-kit-fast | GPUI Fast 0.1.14 |
| freya-056a5b4 | freya 0.6.0-alpha.1 at `056a5b4` |
| dioxus-skia | Skia renderer |

The tuned editions run by default; the last three run by name or with
`just full`. `run.json` records the exact version and git revision of every
framework.

Apps use built-in widgets and the patterns their docs teach, with framework
defaults for rendering and scrolling. Where a framework lacks a part, the app
builds it by hand and the results mark it. [IDIOMS.md](IDIOMS.md) has the rule
and the list.

## Run

```sh
just build        # apps, editions, runner
just quick        # small matrix
just run          # default matrix, about 2 hours
just full         # all editions, 5 iterations, every size
just plots        # figures for results/latest
```

`just run` takes runner options, such as
`--scenarios app --frameworks zgui,egui --rows 300,3000`. `--resume --out
results/<run>` continues or extends a run.

For valid numbers, set the CPU governor to `performance` and leave the machine
idle; run one matrix at a time. The runner refuses `powersave` unless given
`--allow-noisy-host`. A disturbed measurement is repeated up to twice and
flagged invalid if it stays disturbed.

## Results

`results/<run>/run.json` indexes the run: cells, binary sizes, hand-built
parts, framework versions. Each cell directory
`<scenario>/<framework>/<hz>hz/<rows>/` holds `cell.json`, screenshots,
`startup.json`, the app log and one report per step.

`python3 plots/all.py <run>` writes `summary.csv` and seven figures:

| Figure | Shows |
|---|---|
| `capacity.png` | how many elements each interaction tolerates within a 144 Hz and a 60 Hz frame |
| `response.png` | response time against elements, per interaction |
| `slope.png` | milliseconds added per 1,000 elements |
| `smoothness.png` | slowest frames and missed refreshes against elements |
| `timelines.png` | a fling, frame by frame |
| `baseline.png` | startup, memory, idle CPU, hover and binary size of the small app |
| `list.png` | the virtual list at both sizes |

The [explorer](explorer/README.md) is a web app over the same data.

## Method

- Output: 1280x800, scale 1, 144 Hz. Times are compositor timestamps;
  presentation is the modeled refresh of the fake display.
- A probe sends its input at a planned phase of the refresh and detects the
  response in the pixels of a region. A second input restores the state
  between repeats. Hover takes 3 × 10 samples, other probes 3 × 6.
- Idle apps draw on input without waiting for a frame callback, so input to
  commit does not depend on the input's phase.
- Scrolling uses each framework's own scroll container, scale, smoothing and
  momentum. The analyzer measures the app's content gain first and judges
  steps against it. Glide and fling send four events per refresh. The wheel
  moves in notches and has no missed-refresh count.
- Missed refreshes are those without a new frame while one was due.
- Startup ends at the frame that shows the spec's layout, since some
  frameworks present an empty frame first.
- No accessibility bus by default; `--a11y` runs one, which costs some
  frameworks much more than others.
- fakeland samples CPU time, wakeups, memory and GPU use of the app's process
  tree every 50 ms, and derives CPU and GPU time per frame. Power is
  device-wide and only context.
