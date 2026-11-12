# gui-bench explorer

An interactive explorer of gui-bench runs: Leptos 0.9 (server rendering and hydration through
cargo-leptos) on an axum server that reads `../results/` on demand.

```sh
just -f explorer/justfile dev      # cargo leptos watch on http://127.0.0.1:3080
just -f explorer/justfile serve    # release build, then serve
just -f explorer/justfile check    # fmt, clippy (server and wasm), tests
```

The server takes `--results <dir>` (default `../results`) and `--fonts <dir>` (default
`../plots/fonts`). `results/latest` shows as an alias of its run; runs of an older format show
as unsupported.

## Views

- **Overview**: provenance, report health, and a scorecard of every framework over the key
  metric of each scenario (ratio to the best, geometric mean).
- **Dashboard**: the realistic app. Every interaction at both table sizes in one grid colored by
  refreshes, each interaction from 50 to 1,000 orders, the stages of one interaction (app, GPU,
  wait for the refresh, scanout), CPU per frame, the scroll glide frame by frame, resize, startup,
  footprint, and the screenshots side by side.
- **Frame times**: frame interval percentiles (p50, p99, longest gap) and stale refreshes against
  size, in ms or in refreshes; the frame-time profile of one size; every late frame of every
  iteration on its time axis; and the share of frame time per class of lateness. For scroll
  glides, ticker, spinner and both resize storms.
- **Rows & scroll**, **Table ops**, **Ticker & spinner**, **Typing & search**, **Resize** (plain
  list or dashboard), **Memory & idle**: the scenarios, with any metric against rows,
  per-operation dot plots, variant speedups with bootstrap CIs, churn, the efficiency matrix,
  memory per row, and frame times in place of frame rates.
- **Compare**: one framework (or variant) against another, or the run against another run, as a
  forest plot of ratios with 95% bootstrap intervals.
- **Explore**: any metric of a scenario against any other.
- **Runs**: every run with its provenance (accessibility bus, the invocations that wrote it), and
  the differences of two runs.
- **Cell drawer** (click any point): screenshot, startup, validity checks, iterations, every
  latency repeat, stages, frame timelines, per-key typing, per-configure resize, zgui phases,
  app log.

Every chart zooms (drag a box, ctrl+wheel, double-click resets), switches linear and log,
pins outliers to the edge in focus mode, shows the chosen interval (95% CI, IQR, p5–95, range),
and exports SVG, PNG or CSV. Filters, baseline, compare run and the open cell live in the URL.

Framework versions show as "not recorded" until the runner writes `framework_versions` into
`run.json`: `{"<framework>": {"crate": "iced", "version": "0.14.0", "source": "crates.io"}}`.
