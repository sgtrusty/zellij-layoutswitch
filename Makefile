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

.PHONY: all build install clean debug debug-layout debug-pane debug-kill flush-cache docker-build install-docker

# Default action: build and install
all: build install

# Compile the Rust code for WASI in release mode
# NOTE: Ensure the indented lines below use a physical TAB character
build:
	cargo build --release

# Create the directory if it doesn't exist and move the plugin
install:
	mkdir -p $(DEST_DIR)
	cp $(SOURCE_FILE) $(DEST_FILE)
	@echo "------------------------------------------------"
	@echo "Successfully installed to: $(DEST_FILE)"
	@echo "Zellij KDL path: file:$(DEST_FILE)"
	@echo "------------------------------------------------"

# Build plugin via Docker and extract .wasm + updated Cargo.lock
docker-build:
	DOCKER_BUILDKIT=1 docker build --target export \
		--output type=local,dest=$(DOCKER_EXPORT_DIR) \
		-t $(DOCKER_IMAGE) .
	cp $(DOCKER_EXPORT_DIR)/Cargo.lock Cargo.lock
	@echo "Extracted plugin to $(DOCKER_EXPORT_DIR)/$(PLUGIN_NAME).wasm"
	@echo "Updated Cargo.lock from Docker build"

# Install the docker-built plugin (requires docker-build first)
install-docker:
	mkdir -p $(DEST_DIR)
	cp $(SOURCE_FILE) $(DEST_FILE)
	@echo "Installed docker-built plugin to $(DEST_FILE)"

# Clean the build artifacts
clean:
	cargo clean

# Reload the plugin in the current Zellij session
debug:
	zellij action start-or-reload-plugin file:$(SOURCE_FILE)

# Focus a layout (Default: BASE)
debug-layout:
	zellij pipe -n focus-layout -- "$(L)"

# Focus a pane (Default: Module Editor)
debug-pane:
	zellij pipe -n focus-pane -- "$(P)"

debug-kill:
	zellij pipe -n focus-stop || true 

devel:
	zellij --layout dev.kdl

flush-cache:
	rm -rf /home/${USER}/.cache/zellij/*

logs:
	tail /tmp/zellij-1000/zellij-log/zellij.log -f
