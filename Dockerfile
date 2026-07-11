# ==========================================
# STAGE 1: Non-Root Builder
# ==========================================
FROM rust:latest AS builder

RUN groupadd -g 10001 appgroup && \
    useradd -u 10001 -g appgroup -m -s /bin/bash appuser

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/home/appuser/.cargo \
    PATH=/usr/local/cargo/bin:/home/appuser/.cargo/bin:$PATH

USER appuser
WORKDIR /app

RUN rustup target add wasm32-wasip1
COPY --chown=appuser:appgroup Cargo.toml Cargo.lock ./
COPY --chown=appuser:appgroup src ./src
COPY --chown=appuser:appgroup .cargo ./.cargo
RUN --mount=type=cache,target=/usr/local/cargo/registry,uid=10001,gid=10001 \
    --mount=type=cache,target=/app/target,uid=10001,gid=10001 \
    cargo build --release --target wasm32-wasip1 && \
    cp target/wasm32-wasip1/release/zellij-layoutswitch.wasm /app/zellij-layoutswitch.wasm

# ==========================================
# STAGE 2: Test (host target, no wasm)
# ==========================================
FROM rust:latest AS test

RUN groupadd -g 10001 appgroup && \
    useradd -u 10001 -g appgroup -m -s /bin/bash appuser

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/home/appuser/.cargo \
    PATH=/usr/local/cargo/bin:/home/appuser/.cargo/bin:$PATH

USER appuser
WORKDIR /app

COPY --chown=appuser:appgroup Cargo.toml Cargo.lock ./
COPY --chown=appuser:appgroup src ./src
COPY --chown=appuser:appgroup tests ./tests

# Run tests with INSTA_UPDATE=always to auto-accept snapshots
# No .cargo/config.toml — must compile for host target, not wasm
RUN --mount=type=cache,target=/usr/local/cargo/registry,uid=10001,gid=10001 \
    --mount=type=cache,target=/app/target,uid=10001,gid=10001 \
    INSTA_UPDATE=always cargo test 2>&1 | tee /app/test-output.txt; \
    EXIT=$?; \
    mkdir -p /app/snapshots-out; \
    cp -r /app/tests/snapshots /app/snapshots-out/tests-snapshots 2>/dev/null || true; \
    cp /app/test-output.txt /app/snapshots-out/; \
    exit $EXIT

# ==========================================
# STAGE 3: Coverage
# ==========================================
FROM rust:latest AS coverage

RUN groupadd -g 10001 appgroup && \
    useradd -u 10001 -g appgroup -m -s /bin/bash appuser

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/home/appuser/.cargo \
    PATH=/usr/local/cargo/bin:/home/appuser/.cargo/bin:$PATH

USER appuser
WORKDIR /app

USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev pkg-config cmake && \
    rm -rf /var/lib/apt/lists/*
USER appuser

RUN cargo install cargo-tarpaulin --version 0.37.0

COPY --chown=appuser:appgroup Cargo.toml Cargo.lock ./
COPY --chown=appuser:appgroup src ./src
COPY --chown=appuser:appgroup tests ./tests

RUN INSTA_UPDATE=always cargo tarpaulin \
    --engine llvm \
    --include-files 'src/*' \
    --out html \
    --output-dir /app/coverage \
    --skip-clean 2>&1 | tee /app/coverage-output.txt; \
    EXIT=$?; \
    exit $EXIT

# ==========================================
# STAGE 4: Export
# ==========================================
FROM scratch AS export

COPY --from=builder /app/zellij-layoutswitch.wasm /
