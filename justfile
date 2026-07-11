PLUGIN_NAME := "zellij-layoutswitch"
DOCKER_IMAGE := PLUGIN_NAME + "-builder"
DOCKER_EXPORT_DIR := "target/wasm32-wasip1/release"
SOURCE_FILE := "target/wasm32-wasip1/release/" + PLUGIN_NAME + ".wasm"

L := "BASE"
P := "LOGS"

default:
    @just --list

# ── Docker build ──────────────────────────────────────────────────────

docker:
    DOCKER_BUILDKIT=1 docker build --target export \
        --output type=local,dest={{DOCKER_EXPORT_DIR}} \
        -t {{DOCKER_IMAGE}} .
    @echo "Extracted plugin to {{DOCKER_EXPORT_DIR}}/{{PLUGIN_NAME}}.wasm"

docker-lock:
    DOCKER_BUILDKIT=1 docker build --target builder -t {{DOCKER_IMAGE}} .
    docker create --name {{PLUGIN_NAME}}-lock {{DOCKER_IMAGE}}
    docker cp {{PLUGIN_NAME}}-lock:/app/Cargo.lock Cargo.lock
    docker rm {{PLUGIN_NAME}}-lock
    @echo "Extracted Cargo.lock from Docker"

# ── Docker test & coverage ────────────────────────────────────────────

test:
    DOCKER_BUILDKIT=1 docker build --target test \
        -t {{DOCKER_IMAGE}}-test .
    @echo "Tests passed"

test-extract: test
    ./scripts/test-extract.sh {{DOCKER_IMAGE}}-test

coverage:
    DOCKER_BUILDKIT=1 docker build --target coverage \
        -t {{DOCKER_IMAGE}}-coverage .
    mkdir -p coverage && \
    docker create --name {{DOCKER_IMAGE}}-coverage-tmp {{DOCKER_IMAGE}}-coverage && \
    docker cp {{DOCKER_IMAGE}}-coverage-tmp:/app/coverage ./ && \
    docker rm {{DOCKER_IMAGE}}-coverage-tmp
    @echo "Coverage report written to ./coverage/index.html"

# ── Debugging ─────────────────────────────────────────────────────────

debug:
    zellij action start-or-reload-plugin file:{{SOURCE_FILE}}

debug-layout:
    zellij pipe -n focus-layout -- "{{L}}"

debug-pane:
    zellij pipe -n focus-pane -- "{{P}}"

debug-kill:
    zellij pipe -n focus-stop || true

devel:
    zellij --layout dev.kdl

# ── Maintenance ───────────────────────────────────────────────────────

flush-cache:
    rm -rf {{env_var("HOME")}}/.cache/zellij/*

logs:
    tail -f /tmp/zellij-1000/zellij-log/zellij.log
