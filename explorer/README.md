# gui-bench explorer

An interactive explorer of gui-bench runs: Leptos 0.9 (server rendering and hydration through
cargo-leptos) on an axum server that reads `../results/` on demand.

```sh
just -f explorer/justfile dev      # cargo leptos watch on http://127.0.0.1:3080
just -f explorer/justfile serve    # release build, then serve
just -f explorer/justfile check    # fmt, clippy (server and wasm), tests
```

The server takes `--results <dir>` (default `../results`) and `--fonts <dir>` (default
`../plots/fonts`). `results/latest` shows as an alias of its run. The explorer reads run format 3
(the scenarios `app` and `list`); runs of an older format show as unsupported.

## Data

A cell is one scenario, framework (or edition), refresh rate and row count. The size of an `app`
cell on every size axis is the number of elements the spec counts (`elements`); a `list` cell
uses its rows of data. Cells at the slower rate hold only the pacing steps of a cell that missed
refreshes. The derived numbers follow `../plots`:

- **Response time**: input to the commit of the frame that shows the response, plus the median
  time until the GPU finished it, for every repeat (`<step>.latency.response_ms`).
- **Capacity**: the elements up to which the 95th percentile of the response time stays within
  one frame (interpolated on log axes), and up to which a pacing step misses at most 1 % of the
  refreshes.
- **Cost per 1,000 elements**: the slope of a line through the median response times.
- **Missed refreshes**: refreshes without a new frame over the missed and the presented ones,
  per iteration (`<step>.frame_pacing.missed_share`).

Charts mark what the plots mark: a ring for a report that the host disturbed (`validity`), a
cross at the first size a framework did not run because it was over the response budget, hollow
marks for a part the app builds by hand (`click-menu`: menu, `live`: spinner, the `list`
scenario: list), and dashed lines for editions, which sit next to their framework.

## Views

- **Overview**: how far each framework scales within a 144 Hz and a 60 Hz frame (capacity),
  provenance, report health with the cells that did not run or went over budget, and the builds
  with their binary sizes and hand-built parts.
- **Response time**: every interaction of the app against the elements in the app, on one shared
  scale with the frame budgets, and the cost per 1,000 elements.
- **Smoothness**: slowest frames and missed refreshes of the glide, the fling and the live
  updates, and the resize, against the elements in the app, at the main or the slower rate; and
  the fling frame by frame at one size.
- **Frame times**: any measurement with continuous frames of the app or the list: frame interval
  percentiles and missed refreshes against size, in ms or in refreshes; the frame-time profile of
  one size; every late frame of every iteration on its time axis; and the share of frame time
  per class of lateness.
- **Baseline**: the cost of a small app: startup to the content, memory, work at rest, hover
  response and binary size, and the screenshots side by side.
- **List**: the virtual list at each row count: hover and sort response, the scrolls, memory and
  startup.
- **Compare**: one framework against another, or the run against another run, as a forest plot
  of ratios with 95% bootstrap intervals.
- **Explore**: any metric of a scenario against any other, or against the size.
- **Runs**: every run with its provenance (accessibility bus, the invocations that wrote it), and
  the differences of two runs.
- **Cell drawer** (click any point): outcome (not run, over budget, missed refreshes), hand-built
  parts, screenshots, every launch of the startup, validity checks, iterations, every latency
  repeat with its response time, stages, frame timelines, per-configure resize, zgui phases, app
  log.

Every chart zooms (drag a box, ctrl+wheel, double-click resets), switches linear and log,
pins outliers to the edge in focus mode, shows the chosen interval (95% CI, IQR, p5–95, range),
and exports SVG, PNG or CSV. Filters, baseline, compare run and the open cell live in the URL.

Framework versions show as "not recorded" until the runner writes `framework_versions` into
`run.json`: `{"<framework>": {"crate": "iced", "version": "0.14.0", "source": "crates.io"}}`.

## Container

`explorer/Dockerfile` builds an image that holds the server, the site and the
run data. Build it from the repository root, or with `just image` here:

```sh
docker build -f explorer/Dockerfile -t gui-bench-explorer .
docker run --rm -p 8080:8080 gui-bench-explorer     # http://localhost:8080
```

- **Data.** The image holds every run directory of `results/` except
  `results/old/` and the `dev-*` runs (`explorer/Dockerfile.dockerignore`
  keeps those and the rest of the repository out of the build context). `/`
  opens the run whose `run.json` changed last; `--build-arg LATEST_RUN=<run>`
  names another. New data needs a new image.
- **Listening.** The server listens on `0.0.0.0:8080`; `LEPTOS_SITE_ADDR`
  sets another address. Links, assets and server functions use paths from the
  root of the host and no absolute URLs, so the app works behind a reverse
  proxy or an ingress under its own host name. It does not work under a path
  prefix. TLS ends at the proxy.
- **Probes.** `GET /healthz` answers `200 ok` without reading a run; use it
  for the liveness and readiness probes. The server finishes its open requests
  and exits on SIGTERM.
- **Security context.** The image runs as user 65532 and writes no file: it
  runs with a read-only root filesystem, no capabilities and
  `allowPrivilegeEscalation: false`.
- **Resources.** The server keeps the runs it has opened in memory; a run of
  the default matrix is about 140 MB on disk.

A deployment needs no more than this container:

```yaml
containers:
  - name: explorer
    image: gui-bench-explorer
    ports: [{ containerPort: 8080 }]
    readinessProbe: { httpGet: { path: /healthz, port: 8080 } }
    livenessProbe: { httpGet: { path: /healthz, port: 8080 } }
    securityContext:
      runAsNonRoot: true
      readOnlyRootFilesystem: true
      allowPrivilegeEscalation: false
      capabilities: { drop: [ALL] }
```
