# syntax=docker/dockerfile:1
#
# The gui-bench explorer with the run data inside the image.
#
# Build from the repository root, since the image takes the runs from `results/` and the plot
# fonts from `plots/fonts/` (`explorer/Dockerfile.dockerignore` keeps the rest of the repository
# out of the build context):
#
#     docker build -f explorer/Dockerfile -t gui-bench-explorer .
#     docker run --rm -p 8080:8080 gui-bench-explorer
#
# The image holds every run directory of `results/` except `results/old/` and the `dev-*` runs.
# `--build-arg LATEST_RUN=<run>` picks the run that `/` opens; the default is the run whose
# `run.json` changed last.

# ---- Build: the server binary and the site (wasm, JavaScript, CSS, assets) -------------------
FROM rust:1-bookworm AS build

# The versions of the build tools. cargo-leptos fetches its own wasm-bindgen, dart-sass and
# wasm-opt at build time.
ARG CARGO_LEPTOS_VERSION=0.3.12
ARG CARGO_BINSTALL_VERSION=1.27.0

WORKDIR /src/explorer

# The pinned toolchain of the repository, with the wasm target of the frontend.
COPY rust-toolchain.toml /src/rust-toolchain.toml
RUN rustup toolchain install \
    && rustup target add wasm32-unknown-unknown

RUN curl -fsSL "https://github.com/cargo-bins/cargo-binstall/releases/download/v${CARGO_BINSTALL_VERSION}/cargo-binstall-x86_64-unknown-linux-gnu.tgz" \
        | tar -xz -C /usr/local/cargo/bin \
    && cargo binstall --no-confirm --locked "cargo-leptos@${CARGO_LEPTOS_VERSION}"

COPY explorer/ /src/explorer/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/explorer/target \
    cargo leptos build --release \
    && mkdir -p /out \
    && cp target/release/explorer-server /out/explorer-server \
    && cp -r target/site /out/site

# ---- Data: the runs, and the link that `/` opens ---------------------------------------------
FROM debian:bookworm-slim AS data

ARG LATEST_RUN=""

COPY results/ /data/results/
# The `latest` link of a checkout may point outside the repository. Point it at a run in the
# image: the one named by LATEST_RUN, or the one whose index changed last.
RUN set -eu; cd /data/results; rm -f latest; \
    run="${LATEST_RUN}"; \
    if [ -z "$run" ]; then \
        run="$(ls -t */run.json 2>/dev/null | head -n 1 | cut -d/ -f1)"; \
    fi; \
    if [ -n "$run" ] && [ -f "$run/run.json" ]; then \
        ln -s "$run" latest; echo "latest -> $run"; \
    else \
        echo "no run for the latest link: '${run}'" >&2; exit 1; \
    fi

# ---- Run ---------------------------------------------------------------------------------------
# glibc and libgcc only; runs as the unprivileged user 65532 and writes nothing.
FROM gcr.io/distroless/cc-debian12:nonroot

# Links the image to its repository in a registry that reads the label, such as ghcr.io.
LABEL org.opencontainers.image.source="https://github.com/zortax/gui-bench" \
      org.opencontainers.image.description="The gui-bench explorer with its run data" \
      org.opencontainers.image.licenses="Apache-2.0"

COPY --from=build /out/explorer-server /app/explorer-server
COPY --from=build /out/site /app/site
COPY plots/fonts /app/fonts
COPY --from=data /data/results /data/results

# The server reads its site settings from the environment. It listens on every interface, so a
# published port, a Kubernetes service or an ingress reaches it. It serves from the root of its
# host: put it behind a host name, not behind a path prefix.
ENV LEPTOS_OUTPUT_NAME=explorer \
    LEPTOS_SITE_ROOT=/app/site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:8080 \
    LEPTOS_ENV=PROD

WORKDIR /app
EXPOSE 8080
USER 65532:65532
ENTRYPOINT ["/app/explorer-server", "--results", "/data/results", "--fonts", "/app/fonts"]
