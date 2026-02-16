# Variables
PLUGIN_NAME=layoutswitch
SOURCE_FILE=target/wasm32-wasip1/release/$(PLUGIN_NAME).wasm
DEST_DIR=$(HOME)/.config/zellij/plugins
DEST_FILE=$(DEST_DIR)/$(PLUGIN_NAME).wasm

# CLI Overrides (Usage: make debug-layout L="MyLayout")
L ?= BASE
P ?= Module Editor

.PHONY: all build install clean debug debug-layout debug-pane debug-kill flush-cache

# Default action: build and install
all: build install

# Compile the Rust code for WASI in release mode
# NOTE: Ensure the indented lines below use a physical TAB character
build:
	cargo build --release --target wasm32-wasip1

# Create the directory if it doesn't exist and move the plugin
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
