### Zellij Layout Switcher (Rust WASM Plugin)

This plugin provides a high-performance, flicker-free way to switch between **Zellij Swap Layouts** and focus specific panes. Unlike Bash scripts that parse text dumps, this plugin communicates directly with the Zellij engine via the WASM ABI, making transitions nearly instantaneous.

---

## 🛠 What it does

1. **State-Aware Cycling**: When triggered, it cycles through your `swap_tiled_layouts` until it finds the one you requested (e.g., "standard" or "nav_expanded").
2. **Smart Pane Focusing**: Once the correct layout is active, it automatically finds the pane named **"Module Editor"** (regardless of its internal ID) and gives it focus.
3. **Command-Line Trigger**: It listens for `CustomMessage` events, allowing you to trigger complex UI changes from simple shell commands or Yazi/Vim integrations.

---

## 🚀 Installation & Setup

### 1. Build the Plugin

Ensure you have the Rust WASI target installed:

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
mkdir -p ~/.config/zellij/plugins
cp target/wasm32-wasip1/release/zj-layout-switcher.wasm ~/.config/zellij/plugins/
zellij action reload-plugins 2>/dev/null

```

The compiled file will be at:
`target/wasm32-wasip1/release/zj-layout-switcher.wasm`

### 2. Configure your Zellij Layout

Add the plugin to your `layout.kdl`. It is recommended to load it in a 1-row pane or a hidden pane.

```kdl
layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="file:/path/to/zj-layout-switcher.wasm"
        }
    }
    // ... your tab and swap_tiled_layout definitions
}

```

### 3. Usage in Scripts

To switch to a specific layout and focus your editor, send a message to the plugin from any terminal inside Zellij:

```bash
# Switch to the 'standard' swap layout
zellij command post-message "focus-layout" "standard"

# Switch to the 'nav_expanded' swap layout
zellij command post-message "focus-layout" "nav_expanded"

```

---

## 🔧 Integration Example: `vim-open.sh`

Use this script to instantly jump to your "Standard" layout and open a file in Neovim:

```bash
#!/bin/bash
# Tell the plugin to switch layouts and focus the editor
zellij command post-message "focus-layout" "standard"

# Short delay to allow the layout to settle
sleep 0.05

# Open the file
zellij action write 27 # ESC
zellij action write-chars ":e $1"
zellij action write 13 # Enter

```

---

## 📝 Requirements

* **Zellij**: 0.40.0 or higher.
* **KDL Names**: Your panes must have `name="Module Editor"` defined in your KDL for the focus logic to find them.
* **Rust**: `wasm32-wasip1` target.

