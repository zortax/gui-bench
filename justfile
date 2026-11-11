# Build everything, run the full matrix and plot it.
default: build run plots

# The framework apps under `apps/`.
apps := "zgui gpui-kit gpui-kit-fast freya freya-056a5b4 vizia floem iced egui"

# Memory limits: the runner and the apps it launches, and one app build. Each runs in its own
# systemd scope without swap, so a runaway process is killed there instead of stalling the host.
run_mem := "16G"
build_mem := "24G"
limit := "systemd-run --user --scope -q -p MemorySwapMax=0 -p MemoryMax="

# Build every framework app and the runner in release mode.
build:
    for app in {{apps}}; do just build-app $app; done
    {{limit}}{{build_mem}} cargo build --release -p gui-bench

# Build one framework app, such as `just build-app iced`.
build-app name:
    cd apps/{{name}} && {{limit}}{{build_mem}} cargo build --release

# Run the benchmark matrix, for example `just run --scenarios table,ticker --frameworks zgui`.
run *args:
    cargo build -q --release -p gui-bench
    env -u DISPLAY -u WAYLAND_DISPLAY {{limit}}{{run_mem}} target/release/gui-bench {{args}}

# Run a small matrix as a quick check, for example `just quick --scenarios spinner`.
quick *args:
    just run --quick {{args}}

# Run one scenario for every framework, such as `just scenario table`.
scenario name *args:
    just run --scenarios {{name}} {{args}}

# Plot a run directory into its `plots/` folder.
plots run="results/latest":
    python3 plots/all.py {{run}}

# Check the runner, and run the tests of the shared app code in one app.
check:
    cargo fmt --check
    cargo clippy -p gui-bench -- -D warnings
    cd apps/egui && cargo test --release -q
