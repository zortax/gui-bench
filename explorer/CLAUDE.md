# gui-bench explorer

A web explorer of gui-bench runs: Leptos 0.9 (pinned `=0.9.0-rc1`) with server rendering and
hydration through cargo-leptos and leptos_axum. The server reads `../results/<run>/` on demand;
no run data goes into the wasm. Light theme follows the plots of `../plots/common.py`, dark theme
follows Oldworld. Text is Inter (`assets/type`, a subset served at `/type`); headings and card
titles are Libertinus Sans from `../plots/fonts` (served at `/fonts`).

## Workspace
- `crates/explorer-core`: pure. Wire types of the server functions (`api/`), statistics
  (`stats/`), scale, tick and label math (`scale/`), unit formatting (`units/`). Tested natively.
- `crates/explorer-data`: server only, no Leptos. Run catalog, serde views of the run index and
  the fakeland reports, the run cache with metric columns, the detail loaders of one cell and the
  frame profiles of one matrix point.
- `crates/explorer-app`: components, views, chart kernel and server functions. Features `ssr`
  (server) and `hydrate` (browser).
- `crates/explorer-frontend`: the `hydrate()` entry (cdylib). `crates/explorer-server`: the axum
  binary.
- Every dependency version lives once in `[workspace.dependencies]`; crates inherit with
  `foo.workspace = true`. Add crates with `cargo add`, never a version from memory, latest unless
  a concrete reason forces an older one. Leptos crates stay pinned with `=`.
- Toolchain: the nightly of `../rust-toolchain.toml`. Do not change it.
- `#![forbid(unsafe_code)]` in every crate.

## Modules and files
- `mod.rs` style for directory modules. `mod.rs` declares and re-exports; it holds no logic.
- Modularize by feature, not by kind. A view owns its charts; the chart kernel owns its marks.
- Small files, about 400 lines at most. Split a file before it needs a section-separator comment.
- Express the domain in types (`MetricRef`, `CellKey`, `CellId`, `Stat`, `ScaleKind`,
  `Uncertainty`). Stop before the types cost more than they save.
- `thiserror` below the binaries, typed per module. `anyhow` in `main` only.

## Comments and docs
- Short. ASD-STE100 style: one idea per sentence, active voice, present tense.
- Describe what is. Never describe what was there before, what bug was fixed, or why an
  alternative was rejected. No contrastive negation ("not x, but y").
- Never mention plans, specs or external documents in code comments or rustdoc.
- Every module starts with a one-paragraph `//!` that states its responsibility.

## Leptos
- Components are small and named by what they show. Prefer signals, memos and derived signals
  over effects; effects only sync with the DOM or announce chart state to the card.
- Linkable state lives in the URL query (`state/prefs.rs`, `state::use_param`). Writes replace
  the history entry.
- Data loads through SSR `Resource`s, so the server renders every view with its charts and the
  page carries the data it shows. A view asks only for the metric columns it draws
  (`views/kit/data.rs::WithData`, optionally for some frameworks only); the client caches columns
  per run version, so switching views fetches only new columns. The metric catalog travels on its
  own (`metric_catalog`) for the views that need it. `LocalResource` is for browser-only work.
- Charts render at a width hint on the server (`CardCtx::width_hint`) and measure their real
  width after hydration.
- The browser must build the markup the server rendered. serde_json parses floats exactly
  (`float_roundtrip`), so values match bit for bit after the trip; still snap tiny remainders
  (shares, differences) to zero before they decide whether a mark exists.
- A view that looks at one size and rate keeps them in the URL through
  `views/kit::use_scope`, so the server renders the same scope as the browser.
- Charts stay cheap on hover: a chart wraps its model in one `Memo`, reads it with `with`/`read`
  (never `get` in event handlers), and highlights through reactive attributes on series, never by
  re-rendering the SVG. Series colors are CSS variables on the series group.
- A control that changes what a `WithData` loads lives outside that `WithData`: a reload
  disposes everything inside it, and a control disposed during its own event panics. Shared
  widgets read the signals they get with `try_get`, and conditional parts are closures with
  `try_get` instead of `<Show>`.
- Look cells up by id with `get`: a live run can send columns with cells newer than the overview
  of the page.
- Server functions use `#[server(input = GetUrl)]` and wire types from `explorer_core::api`
  (qualified as `wire::X`, since the macro names a struct after the function).
- Closures with type annotations in `view!` attributes need braces: `view={move |o: Arc<T>| ..}`.
- No JS chart libraries. Charts are SVG from `chart/`: `XyPlot`, `DotPlot`, `Heatmap` inside a
  `ChartCard`; views build models with `views/kit::Build`.
- Build the server through cargo-leptos (`just build`): the wasm file name is fixed at compile
  time from `LEPTOS_OUTPUT_NAME`. Do not build `explorer-app` with plain `cargo build` between
  cargo-leptos builds: the two share artifacts with other features, and incremental objects go
  stale (linker: undefined hidden symbol). `CARGO_INCREMENTAL=0` avoids it; after it happened,
  `cargo clean -p explorer-app -p explorer-server`.

## Styling
- SCSS in `style/`, one partial per feature, BEM classes with the `gx-` prefix, `data-*`
  attributes for state.
- Components use tokens only (`style/_tokens.scss`). Framework colors are CSS variables per slot
  (`style/_palette.scss`, named by `palette::mark/soft/ink`), so both themes work without Rust.
- Compact and flat: planes differ by tone, alpha borders only on floating surfaces, motion under
  120 ms.

## Verify
- `just check`: fmt, clippy with `-D warnings` for `ssr` and for `hydrate` on wasm32, tests.
- `just dev` runs `cargo leptos watch` on <http://127.0.0.1:3080>.
- Look at changes in a browser in the light and the dark theme.
