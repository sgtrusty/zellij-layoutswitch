use zellij_tile::prelude::*;
use serde::de::DeserializeOwned;

use crate::worker::LAYOUT_WORKER;

// ── Message name constants ──────────────────────────────────────────

pub(crate) const MSG_PANE_UPDATE: &str = "pane-update";
pub(crate) const MSG_TAB_UPDATE: &str = "tab-update";
pub(crate) const MSG_PERMISSION_RESULT: &str = "permission-result";
pub(crate) const MSG_EXECUTE_ACTION: &str = "execute-action";
pub(crate) const MSG_FOCUS_PANE: &str = "do-focus-pane";
pub(crate) const MSG_UPDATE_STATUS: &str = "update-status";

// ── Action payload constants ────────────────────────────────────────

pub(crate) const ACTION_NEXT_SWAP_LAYOUT: &str = "next-swap-layout";
pub(crate) const ACTION_CLOSE_SELF: &str = "close-self";
pub(crate) const ACTION_HIDE_SELF: &str = "hide-self";

// ── Message struct ──────────────────────────────────────────────────

/// An internal message exchanged between [`State`](crate::state::State) and
/// [`LayoutWorker`](crate::worker::LayoutWorker).
///
/// Construct via the named factory methods, then call [`encode`](Self::encode)
/// to turn it into a [`PluginMessage`] or [`act`](Self::act) to dispatch it on
/// the plugin thread.
pub(crate) struct Message {
    name: String,
    payload: String,
}

impl Message {
    pub(crate) fn new(name: impl Into<String>, payload: impl Into<String>) -> Self {
        Self { name: name.into(), payload: payload.into() }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn payload(&self) -> &str { &self.payload }

    /// Decode the JSON payload into a concrete type.
    pub fn decode<T: DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_str(&self.payload).ok()
    }

    /// Serialize into a [`PluginMessage`] for the plugin thread (no worker).
    pub fn to_plugin(&self) -> PluginMessage {
        PluginMessage {
            name: self.name.clone(),
            payload: self.payload.clone(),
            worker_name: None,
        }
    }

    /// Serialize into a [`PluginMessage`] addressed to the layout worker.
    pub fn to_worker(&self) -> PluginMessage {
        PluginMessage {
            name: self.name.clone(),
            payload: self.payload.clone(),
            worker_name: Some(LAYOUT_WORKER.to_string()),
        }
    }

    /// Dispatch this message as an action on the plugin thread.
    ///
    /// Only meaningful for [`MSG_EXECUTE_ACTION`] and [`MSG_FOCUS_PANE`];
    /// other message types are silently ignored.
    pub fn act(&self) {
        match self.name.as_str() {
            MSG_EXECUTE_ACTION => match self.payload.as_str() {
                ACTION_NEXT_SWAP_LAYOUT => next_swap_layout(),
                ACTION_CLOSE_SELF => close_self(),
                ACTION_HIDE_SELF => hide_self(),
                _ => (),
            },
            MSG_FOCUS_PANE => {
                if let Ok(id) = self.payload.parse::<u32>() {
                    focus_terminal_pane(id, false, false);
                }
            }
            _ => (),
        }
    }

    // ── Factory constructors ────────────────────────────────────────

    /// State → Worker: a pane manifest update.
    pub fn pane_update(manifest: &PaneManifest) -> Option<Self> {
        serde_json::to_string(manifest).ok().map(|p| Self::new(MSG_PANE_UPDATE, p))
    }

    /// State → Worker: a tab info update.
    pub fn tab_update(tabs: &[TabInfo]) -> Option<Self> {
        serde_json::to_string(tabs).ok().map(|p| Self::new(MSG_TAB_UPDATE, p))
    }

    /// State → Worker: a permission request result.
    pub fn permission_result(result: &PermissionStatus) -> Option<Self> {
        serde_json::to_string(result).ok().map(|p| Self::new(MSG_PERMISSION_RESULT, p))
    }

    /// Worker → Plugin: execute a named action (e.g. `next-swap-layout`).
    pub fn execute_action(action: &str) -> Self {
        Self::new(MSG_EXECUTE_ACTION, action)
    }

    /// Worker → Plugin: focus a terminal pane by ID.
    pub fn focus_pane(id: u32) -> Self {
        Self::new(MSG_FOCUS_PANE, id.to_string())
    }

    /// Worker → Plugin: update the status bar text.
    pub fn update_status(status: String) -> Self {
        Self::new(MSG_UPDATE_STATUS, status)
    }
}
