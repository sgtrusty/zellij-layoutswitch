# Variables
PLUGIN_NAME=layoutswitch
SOURCE_FILE=target/wasm32-wasip1/release/$(PLUGIN_NAME).wasm
DEST_DIR=$(HOME)/.config/zellij/plugins
DEST_FILE=$(DEST_DIR)/$(PLUGIN_NAME).wasm

.PHONY: all build install clean

# Default action: build and install
all: build install

# Compile the Rust code for WASI in release mode
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
