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

/// Re-exports for integration tests only.
#[doc(hidden)]
#[allow(unused_imports)]
pub mod test_support {
    pub use crate::message::{
        Message, MSG_PANE_UPDATE, MSG_TAB_UPDATE, MSG_PERMISSION_RESULT,
        MSG_EXECUTE_ACTION, MSG_FOCUS_PANE, MSG_UPDATE_STATUS,
        ACTION_NEXT_SWAP_LAYOUT, ACTION_CLOSE_SELF, ACTION_HIDE_SELF,
    };
    pub use crate::worker::MAX_LAYOUT_RETRIES;
}
