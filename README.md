# gui-bench

Benchmarks of Rust GUI frameworks under
[fakeland](../fakeland), a headless Wayland compositor that fakes the display
and measures every frame from input to presentation.

Each framework renders the same window: a scroll container with N rows, all
of them real elements (no virtualization). The matrix runs every framework at
every refresh rate and row count and measures:

- **Startup:** process spawn to the first presented frame of the full list.
- **Hover latency:** the pointer enters a row; the time until the frame with
  the hover color is on screen, split into app work (input to commit), GPU,
  the wait for the refresh, and presentation.
- **Scroll smoothness:** a finger glide at 800 px/s; the content
  displacement of every presented frame against the input, missed refreshes,
  delivered frame rate and tracking.
- **zgui frame phases:** zgui reports each frame and its phases (style,
  layout, paint, record, submit) through fakeland app marks.

## Frameworks

| Framework | App | Source |
|---|---|---|
| zgui | `apps/zgui-rows` | path dependency on `../zgui-fakeland-marks` (branch `feat/fakeland-marks`), feature `fakeland-marks` |
| gpui-kit | `apps/gpui-kit-rows` | `gpui-kit` 0.9.1 from crates.io (GPUI with gpui-base and gpui-component) |

Each app is its own cargo workspace, so the two GUI stacks never share a
dependency resolve. The runner depends on fakeland by path (`../fakeland`).

## Run

```sh
just build                 # the apps and the runner, release
just run                   # the full matrix, about 15 minutes
just quick                 # 100 and 3000 rows at 60 Hz
just plots                 # plots of results/latest
```

`just run` takes runner options, for example
`just run --frameworks zgui --rows 1000,10000 --hz 144`. Results go to
`results/<run>/`: `run.json` indexes the cells, and every cell directory
`<framework>/<hz>hz/<rows>/` holds the reports as JSON (fakeland bench
reports with per-iteration and per-sample data). `results/latest` links to
the newest run.

## Plots

`plots/` holds Python scripts (matplotlib only). `plots/all.py <run>` writes
every plot into `<run>/plots/` and a tidy `<run>/summary.csv`:

- `hover_latency.png`: hover latency against rows, median and 5th to 95th
  percentile, one panel per refresh rate.
- `hover_stages.png`: the median of each latency stage per framework and row
  count.
- `scroll.png`: smoothness, delivered frame rate, missed refreshes and
  tracking against rows.
- `startup.png`: spawn to first presented frame against rows.
- `zgui_phases.png`: zgui CPU time per frame phase against rows.

Each script also runs on its own: `python3 plots/scroll.py results/latest`.

## Method notes

- The output is 1280x800 at scale 1. Rows are 32 px tall and start at the top
  of the window. The hover probe enters row 5 from row 1, at planned phases
  of the refresh, and detects the response by the pixels of row 5.
- Scroll uses the touchpad (finger) source without kinetic scrolling, at four
  events per refresh, so every frame consumes the same scroll. Motion
  estimation reads the rows above the pointer, since the hovered row under
  the pointer does not move with the content. Lists shorter than a glide
  (10 rows) have no scroll results.
- GPUI on Wayland multiplies touchpad distances by 3 and turns wheel steps
  into lines; the gpui-kit app undoes both so content moves 1:1 with the
  input. zgui animates wheel steps; the zgui app moves the list at once. Both
  apps show no animation on hover.
- Without app marks, fakeland assigns each input to the next commit by time,
  so per-frame expected scroll steps jitter by an event for gpui-kit. The
  totals and the pixel-based measures are unaffected.
- Times are compositor timestamps. A run is invalid when the compositor wakes
  late; each report carries its validity and the host environment.
