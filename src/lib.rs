//! A Zellij plugin for switching between swap layouts and focusing panes by title.
//!
//! # Architecture
//! The plugin is split into two communicating parts:
//! - [`State`] runs on the plugin thread, handles Zellij events, and forwards
//!   commands to the worker.
//! - [`LayoutWorker`] runs in a background worker thread and processes pane/tab
//!   state, resolves pane titles to IDs, and cycles through swap layouts.

mod layout;
mod message;
pub mod output_port;
mod state;
mod worker;

pub use output_port::{AbiPort, OutputPort, RecordingPort};
pub use state::State;
pub use worker::{LayoutWorker, LAYOUT_WORKER};

pub(crate) fn log(msg: impl std::fmt::Display) {
    eprintln!("[plugin-layoutswitch] {}", msg);
}

/// Stub for the Zellij ABI symbol `host_run_plugin_command`.
/// When compiling natively for tests, zellij-tile's own stub may not link
/// correctly due to Docker layer caching mixing wasm32/host artifacts.
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub unsafe extern "C" fn host_run_plugin_command() {}

// Integration-style tests kept inside the crate (under `src/tests/`) so the
// internal items they exercise can stay `pub(crate)` instead of being exposed
// as `pub` + `#[doc(hidden)]` in the public API.
#[cfg(test)]
#[path = "tests/layout_flow.rs"]
mod layout_flow;
#[cfg(test)]
#[path = "tests/state_routing.rs"]
mod state_routing;
#[cfg(test)]
#[path = "tests/worker_dispatch.rs"]
mod worker_dispatch;
