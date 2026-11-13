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

# The settings of the size builds: optimize for size, link-time optimization over the whole
# program in one unit, abort on panic, no debug information and no symbols. The standard library
# is built the same way, which needs a nightly toolchain with the `rust-src` component.
min_profile := "CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_STRIP=symbols CARGO_PROFILE_RELEASE_DEBUG=0"
min_std := "-Zbuild-std=std,panic_abort -Zbuild-std-features=optimize_for_size"

# Build every app and edition for the smallest binary. The benchmarks never run these builds:
# the runner only records their sizes next to those of the builds it measures.
build-min:
    for app in {{apps}}; do just build-min-app $app; done
    just build-min-app gpui-kit min-tuned "--features tuned"
    just build-min-app iced min-tuned "--features tuned"
    just build-min-app dioxus min-skia "--no-default-features --features skia"

# Build one app for the smallest binary into `target/<dir>/<app>-bench`.
build-min-app name dir="min" features="":
    cd apps/{{name}} && if [ -f ui/package.json ]; then (cd ui && npm ci && npm run build); fi
    cd apps/{{name}} && host=$(rustc -vV | sed -n 's/^host: //p') && \
        env {{min_profile}} {{limit}}{{build_mem}} cargo build --release {{features}} \
            --target-dir target/{{dir}} --target $host {{min_std}} && \
        cp target/{{dir}}/$host/release/{{name}}-bench target/{{dir}}/{{name}}-bench

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

# The registry name of the explorer image, and its tag for the checked-out commit: the short
# revision, with `-dirty` when the tree has changes.
image_name := "ghcr.io/zortax/gui-bench"
image_tag := `git describe --always --dirty`

# Build the explorer image with the runs of `results/` inside (not `old/` and `dev-*`), tagged
# `latest` and with the commit.
image *args:
    docker build -f explorer/Dockerfile -t {{image_name}}:latest -t {{image_name}}:{{image_tag}} {{args}} .

# Build the explorer image and push both tags. Needs `docker login ghcr.io`.
image-push: image
    docker push {{image_name}}:{{image_tag}}
    docker push {{image_name}}:latest

# Check the runner, and run the tests of the shared app code in one app.
check:
    cd runner && cargo fmt --check
    cd runner && cargo clippy -- -D warnings
    cd apps/egui && cargo test --release -q
