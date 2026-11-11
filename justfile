# Build everything, run the full matrix and plot it.
default: build run plots

# Build the rows apps and the runner in release mode.
build: build-zgui build-gpui-kit
    cargo build --release -p gui-bench

# Build the zgui rows app with fakeland frame marks.
build-zgui:
    cd apps/zgui-rows && cargo build --release

# Build the gpui-kit rows app.
build-gpui-kit:
    cd apps/gpui-kit-rows && cargo build --release

# Run the benchmark matrix, for example `just run --frameworks zgui --hz 144`.
run *args:
    env -u DISPLAY -u WAYLAND_DISPLAY cargo run -q --release -p gui-bench -- {{args}}

# Run a small matrix as a quick check.
quick:
    just run --quick

# Plot a run directory into its `plots/` folder.
plots run="results/latest":
    python3 plots/all.py {{run}}

# Check the runner.
check:
    cargo fmt --check
    cargo clippy -p gui-bench -- -D warnings
