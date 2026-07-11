use std::cell::RefCell;
use std::rc::Rc;
use zellij_tile::prelude::*;

use crate::message::Message;

/// Abstraction over the Zellij ABI output calls.
///
/// Production uses [`AbiPort`] which delegates to the real `zellij_tile` free
/// functions. Tests inject a [`RecordingPort`] that captures all emitted
/// messages for assertion.
pub trait OutputPort {
    /// Send a [`PluginMessage`] to the plugin thread (State).
    fn post_to_plugin(&self, msg: PluginMessage);
    /// Send a [`PluginMessage`] to the worker thread.
    fn post_to(&self, msg: PluginMessage);
    /// Dispatch a [`Message`] as an action (e.g. `next_swap_layout`, `focus_terminal_pane`).
    fn act(&self, msg: &Message);
}

/// Production port that delegates to the real `zellij_tile` ABI free functions.
pub struct AbiPort;

impl OutputPort for AbiPort {
    fn post_to_plugin(&self, msg: PluginMessage) {
        post_message_to_plugin(msg);
    }
    fn post_to(&self, msg: PluginMessage) {
        post_message_to(msg);
    }
    fn act(&self, msg: &Message) {
        msg.act_abi();
    }
}

/// Test port that records every emitted message for later assertion.
///
/// Uses interior mutability so it can be used through `&dyn OutputPort`.
#[derive(Default, Clone, Debug)]
pub struct RecordingPort {
    pub plugin_messages: Rc<RefCell<Vec<PluginMessage>>>,
    pub worker_messages: Rc<RefCell<Vec<PluginMessage>>>,
    pub actions: Rc<RefCell<Vec<String>>>,
}

impl OutputPort for RecordingPort {
    fn post_to_plugin(&self, msg: PluginMessage) {
        self.plugin_messages.borrow_mut().push(msg);
    }
    fn post_to(&self, msg: PluginMessage) {
        self.worker_messages.borrow_mut().push(msg);
    }
    fn act(&self, msg: &Message) {
        self.actions.borrow_mut().push(msg.payload().to_string());
    }
}

impl RecordingPort {
    /// Return a snapshot of all plugin-bound messages (consumed).
    pub fn take_plugin_messages(&self) -> Vec<PluginMessage> {
        std::mem::take(&mut *self.plugin_messages.borrow_mut())
    }

    /// Return a snapshot of all worker-bound messages (consumed).
    pub fn take_worker_messages(&self) -> Vec<PluginMessage> {
        std::mem::take(&mut *self.worker_messages.borrow_mut())
    }

    /// Return a snapshot of all action payloads (consumed).
    pub fn take_actions(&self) -> Vec<String> {
        std::mem::take(&mut *self.actions.borrow_mut())
    }
}

// ── Thread-local port access ────────────────────────────────────────

thread_local! {
    static PORT: RefCell<Option<Box<dyn OutputPort>>> = RefCell::new(None);
}

/// Access the currently installed [`OutputPort`].
///
/// Returns a thin [`DefaultPort`] wrapper when no port has been installed
/// (i.e. in normal Zellij operation where `set_port` was never called).
pub fn output_port() -> DefaultPort {
    DefaultPort
}

/// A zero-sized type that delegates to the thread-local port if one is set,
/// otherwise falls through to [`AbiPort`].
pub struct DefaultPort;

impl OutputPort for DefaultPort {
    fn post_to_plugin(&self, msg: PluginMessage) {
        PORT.with(|p| {
            if let Some(ref port) = *p.borrow() {
                port.post_to_plugin(msg);
            } else {
                AbiPort.post_to_plugin(msg);
            }
        });
    }
    fn post_to(&self, msg: PluginMessage) {
        PORT.with(|p| {
            if let Some(ref port) = *p.borrow() {
                port.post_to(msg);
            } else {
                AbiPort.post_to(msg);
            }
        });
    }
    fn act(&self, msg: &Message) {
        PORT.with(|p| {
            if let Some(ref port) = *p.borrow() {
                port.act(msg);
            } else {
                AbiPort.act(msg);
            }
        });
    }
}

/// Install an [`OutputPort`] for the current thread. Used by tests.
pub fn set_port(port: Box<dyn OutputPort>) {
    PORT.with(|p| *p.borrow_mut() = Some(port));
}

/// Remove the installed [`OutputPort`], reverting to ABI fallback.
pub fn clear_port() {
    PORT.with(|p| *p.borrow_mut() = None);
}

