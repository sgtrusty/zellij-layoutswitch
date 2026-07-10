# Variables
PLUGIN_NAME=zellij-layoutswitch
SOURCE_FILE=target/wasm32-wasip1/release/$(PLUGIN_NAME).wasm
DEST_DIR=$(HOME)/.config/zellij/plugins
DEST_FILE=$(DEST_DIR)/$(PLUGIN_NAME).wasm
DOCKER_IMAGE=$(PLUGIN_NAME)-builder
DOCKER_EXPORT_DIR=target/wasm32-wasip1/release

# CLI Overrides (Usage: make debug-layout L="MyLayout")
L ?= BASE
P ?= LOGS

.PHONY: help build docker clean install debug debug-layout debug-pane debug-kill devel flush-cache logs

# `make` with no target shows this help
help:
	@echo "Available targets:"
	@echo "  make build         Build the plugin with cargo (wasm32-wasip1)"
	@echo "  make docker        Build the plugin via Docker"
	@echo "  make install       Install the built plugin to $(DEST_DIR)"
	@echo "  make debug         Reload the plugin in the current Zellij session"
	@echo "  make debug-layout  Focus a layout  (L=NAME, default: BASE)"
	@echo "  make debug-pane    Focus a pane    (P=NAME, default: LOGS)"
	@echo "  make debug-kill    Close the plugin"
	@echo "  make devel         Launch Zellij with dev.kdl layout"
	@echo "  make clean         Remove build artifacts"
	@echo "  make flush-cache   Remove Zellij cache"
	@echo "  make logs          Tail Zellij log"

# Compile the Rust code for WASI in release mode
build:
	cargo build --release --target wasm32-wasip1

# Build the plugin via Docker and extract the .wasm
docker:
	DOCKER_BUILDKIT=1 docker build --target export \
		--output type=local,dest=$(DOCKER_EXPORT_DIR) \
		-t $(DOCKER_IMAGE) .
	@echo "Extracted plugin to $(DOCKER_EXPORT_DIR)/$(PLUGIN_NAME).wasm"

# Install the plugin to the local zellij plugins dir
install:
	mkdir -p $(DEST_DIR)
	cp $(SOURCE_FILE) $(DEST_FILE)
	@echo "------------------------------------------------"
	@echo "Successfully installed to: $(DEST_FILE)"
	@echo "Zellij KDL path: file:$(DEST_FILE)"
	@echo "------------------------------------------------"

# Clean the build artifacts
clean:
	cargo clean

# Reload the plugin in the current Zellij session
debug:
	zellij action start-or-reload-plugin file:$(SOURCE_FILE)

# Focus a layout (Default: BASE)
debug-layout:
	zellij pipe -n focus-layout -- "$(L)"

# Focus a pane (Default: LOGS)
debug-pane:
	zellij pipe -n focus-pane -- "$(P)"

# Close the plugin
debug-kill:
	zellij pipe -n focus-stop || true

# Launch Zellij with the dev layout
devel:
	zellij --layout dev.kdl

# Remove Zellij cache
flush-cache:
	rm -rf $(HOME)/.cache/zellij/*

# Tail Zellij log
logs:
	tail -f /tmp/zellij-1000/zellij-log/zellij.log
