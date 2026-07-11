# Zellij Layout Switcher

![GitHub Repo stars](https://img.shields.io/github/stars/sgtrusty/zellij-layoutswitch?style=flat&logo=github)
![License](https://img.shields.io/github/license/sgtrusty/zellij-layoutswitch)
![Rust](https://img.shields.io/badge/rust-wasm32--wasip1-orange?logo=rust)
![Zellij Plugin](https://img.shields.io/badge/zellij-plugin-purple?logo=zellij)
[![Awesome Zellij](https://img.shields.io/badge/Awesome-Zellij-blue?style=flat)](https://github.com/zellij-org/awesome-zellij/)

> 🎉 Featured in [Awesome Zellij](https://github.com/zellij-org/awesome-zellij/) on Feb 17, 2026

<br />

A high-performance Rust WASM plugin for Zellij that enables instant, state-aware swapping of **Swap Layouts** and precise **Pane Focusing**.

---

## 🛠 Features

- **Layout Cycling**: Transitions between `swap_tiled_layouts` (e.g., from "compact" to "expanded") by name.

- **Targeted Focusing**: Jumps to specific panes by their title (e.g., your "Module Editor") across any layout.

- **ABI-Native**: Uses the Zellij WASM ABI for near-zero latency.

- **Automation-Ready**: Listen for `CustomMessage` or `Pipe` triggers from external scripts, Neovim, or Yazi.

---

## 🚀 Getting Started

### 1. Build & Install

Ensure you have the Rust WASI target installed:
`rustup target add wasm32-wasip1`

Using the provided **Makefile**, you can compile and move the plugin to your local config in one go:

```bash
make          # Builds release WASM and installs to ~/.config/zellij/plugins/
# OR
make build    # Just compile
```

### 2. Deployment

To use the plugin in your permanent setup, add it to your `layout.kdl`:

```kdl
load_plugins {
    "file:target/wasm32-wasip1/release/zellij-layoutswitch.wasm"
}

layout {
    children
}
```

---

## 🧪 Development & Testing

For active development, you can use a dedicated KDL layout to automate the **Build → Reload → Debug** cycle.

### Manual Test Run

If you want to run the plugin instantly without a layout file:

```bash
zellij action start-or-reload-plugin file:target/wasm32-wasip1/release/layoutswitch.wasm
```

### Dev Environment

Run this via: `zellij --layout dev.kdl`

---

## ⌨️ Usage (The API)

The plugin listens for two primary commands via Zellij's messaging system:

### 1. `focus-layout`

Cycles through swap layouts until the active one matches your target.

```bash
zellij pipe -n focus-layout -- "nav_expanded"
```

### 2. `focus-pane`

Finds a terminal pane with a specific name and gives it focus.

```bash
zellij pipe -n focus-pane -- "Terminal 1"
```

---

## 🧹 Maintenance

```bash
make clean    # Remove build artifacts
```
