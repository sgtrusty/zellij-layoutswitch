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
mod state;
mod worker;

pub use state::State;
pub use worker::{LayoutWorker, LAYOUT_WORKER};

pub(crate) fn log(msg: impl std::fmt::Display) {
    eprintln!("[plugin-layoutswitch] {}", msg);
}
