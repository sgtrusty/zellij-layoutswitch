# Variables
PLUGIN_NAME=zellij-layoutswitch
SOURCE_FILE=target/wasm32-wasip1/release/$(PLUGIN_NAME).wasm
DEST_DIR=$(HOME)/.config/zellij/plugins
DEST_FILE=$(DEST_DIR)/$(PLUGIN_NAME).wasm

.PONY: help build install uninstall clean

# `make` with no target shows this help
help:
	@echo "Available targets:"
	@echo "  make build         Build the plugin with cargo (wasm32-wasip1)"
	@echo "  make install       Install the built plugin to $(DEST_DIR)"
	@echo "  make uninstall     Remove the installed plugin"
	@echo "  make clean         Remove build artifacts"
	@echo ""
	@echo "Docker, debugging, and CI targets are in the Justfile (just --list)."

build:
	cargo build --release --target wasm32-wasip1

install:
	mkdir -p $(DEST_DIR)
	cp $(SOURCE_FILE) $(DEST_FILE)
	@echo "Installed to $(DEST_FILE)"

uninstall:
	rm -f $(DEST_FILE)
	@echo "Removed $(DEST_FILE)"

clean:
	cargo clean
