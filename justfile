# Build everything, run the full matrix and plot it.
default: build run plots

# The framework apps under `apps/`.
apps := "zgui gpui-kit gpui-kit-fast freya freya-056a5b4 vizia floem iced egui dioxus tauri"

# Memory limits: the runner and the apps it launches, and one app build. Each runs in its own
# systemd scope without swap, so a runaway process is killed there instead of stalling the host.
run_mem := "16G"
build_mem := "24G"
limit := "systemd-run --user --scope -q -p MemorySwapMax=0 -p MemoryMax="

# Build every framework app and the runner in release mode.
build:
    for app in {{apps}}; do just build-app $app; done
    just build-editions
    cd runner && {{limit}}{{build_mem}} cargo build --release

# Build one framework app, such as `just build-app iced`. An app with a web page in `ui/` (the
# Tauri app) builds the page first, since the binary embeds it.
build-app name:
    cd apps/{{name}} && if [ -f ui/package.json ]; then (cd ui && npm ci && npm run build); fi
    cd apps/{{name}} && {{limit}}{{build_mem}} cargo build --release

# Build the editions that are feature builds of an app: GPUI and iced with their opt-in caching,
# and Dioxus Native with the Skia renderer.
build-editions:
    cd apps/gpui-kit && {{limit}}{{build_mem}} cargo build --release --features tuned --target-dir target/tuned
    cd apps/iced && {{limit}}{{build_mem}} cargo build --release --features tuned --target-dir target/tuned
    cd apps/dioxus && {{limit}}{{build_mem}} cargo build --release --no-default-features --features skia --target-dir target/skia

# Run the benchmark matrix, for example `just run --scenarios app --frameworks zgui,egui`.
run *args:
    cd runner && cargo build -q --release
    env -u DISPLAY -u WAYLAND_DISPLAY {{limit}}{{run_mem}} runner/target/release/gui-bench {{args}}

# Run the whole matrix for a publication: every edition, more iterations, every size.
full *args:
    just run --full {{args}}

# Run a small matrix as a quick check, for example `just quick --scenarios list`.
quick *args:
    just run --quick {{args}}

# Run one scenario for every framework, such as `just scenario list`.
scenario name *args:
    just run --scenarios {{name}} {{args}}

# Plot a run directory into its `plots/` folder.
plots run="results/latest":
    python3 plots/all.py {{run}}

# Check the runner, and run the tests of the shared app code in one app.
check:
    cd runner && cargo fmt --check
    cd runner && cargo clippy -- -D warnings
    cd apps/egui && cargo test --release -q
