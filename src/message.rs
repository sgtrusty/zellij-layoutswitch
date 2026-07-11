use zellij_tile::prelude::*;
use serde::de::DeserializeOwned;

use crate::worker::LAYOUT_WORKER;
use crate::output_port::OutputPort;

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
/// Construct via the named factory methods, then call [`to_plugin`](Self::to_plugin)
/// to turn it into a [`PluginMessage`] or [`act`](Self::act) to dispatch it on
/// the plugin thread.
pub(crate) struct Message {
    name: String,
    payload: String,
}

impl Message {
    pub fn new(name: impl Into<String>, payload: impl Into<String>) -> Self {
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

    /// Dispatch this message as an action via the current [`OutputPort`](crate::output_port::OutputPort).
    pub fn act(&self) {
        crate::output_port::output_port().act(self);
    }

    /// Dispatch this message as a direct ABI call (no port indirection).
    pub(crate) fn act_abi(&self) {
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

    /// State -> Worker: a pane manifest update.
    pub fn pane_update(manifest: &PaneManifest) -> Option<Self> {
        serde_json::to_string(manifest).ok().map(|p| Self::new(MSG_PANE_UPDATE, p))
    }

    /// State -> Worker: a tab info update.
    pub fn tab_update(tabs: &[TabInfo]) -> Option<Self> {
        serde_json::to_string(tabs).ok().map(|p| Self::new(MSG_TAB_UPDATE, p))
    }

    /// State -> Worker: a permission request result.
    pub fn permission_result(result: &PermissionStatus) -> Option<Self> {
        serde_json::to_string(result).ok().map(|p| Self::new(MSG_PERMISSION_RESULT, p))
    }

    /// Worker -> Plugin: execute a named action (e.g. `next-swap-layout`).
    pub fn execute_action(action: &str) -> Self {
        Self::new(MSG_EXECUTE_ACTION, action)
    }

    /// Worker -> Plugin: focus a terminal pane by ID.
    pub fn focus_pane(id: u32) -> Self {
        Self::new(MSG_FOCUS_PANE, id.to_string())
    }

    /// Worker -> Plugin: update the status bar text.
    pub fn update_status(status: String) -> Self {
        Self::new(MSG_UPDATE_STATUS, status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors() {
        let msg = Message::new("foo", "bar");
        assert_eq!(msg.name(), "foo");
        assert_eq!(msg.payload(), "bar");
    }

    #[test]
    fn decode_valid_json() {
        let msg = Message::new("test", r#"{"key":"value"}"#);
        let decoded: Option<serde_json::Value> = msg.decode();
        assert!(decoded.is_some());
        assert_eq!(decoded.unwrap()["key"], "value");
    }

    #[test]
    fn decode_invalid_json() {
        let msg = Message::new("test", "not json");
        assert!(msg.decode::<serde_json::Value>().is_none());
    }

    #[test]
    fn to_plugin_has_no_worker_name() {
        let pm = Message::new("n", "p").to_plugin();
        assert_eq!(pm.name, "n");
        assert_eq!(pm.payload, "p");
        assert!(pm.worker_name.is_none());
    }

    #[test]
    fn to_worker_has_layout_worker_name() {
        let pm = Message::new("n", "p").to_worker();
        assert_eq!(pm.worker_name.as_deref(), Some("layout"));
    }

    #[test]
    fn factory_execute_action() {
        let msg = Message::execute_action("my-action");
        assert_eq!(msg.name(), MSG_EXECUTE_ACTION);
        assert_eq!(msg.payload(), "my-action");
    }

    #[test]
    fn factory_focus_pane() {
        let msg = Message::focus_pane(99);
        assert_eq!(msg.name(), MSG_FOCUS_PANE);
        assert_eq!(msg.payload(), "99");
    }

    #[test]
    fn factory_update_status() {
        let msg = Message::update_status("hello".into());
        assert_eq!(msg.name(), MSG_UPDATE_STATUS);
        assert_eq!(msg.payload(), "hello");
    }

    #[test]
    fn factory_pane_update_roundtrip() {
        let manifest = PaneManifest { panes: Default::default() };
        let msg = Message::pane_update(&manifest).unwrap();
        assert_eq!(msg.name(), MSG_PANE_UPDATE);
        let decoded: Option<PaneManifest> = msg.decode();
        assert!(decoded.is_some());
    }

    #[test]
    fn factory_tab_update_roundtrip() {
        let msg = Message::tab_update(&[]).unwrap();
        assert_eq!(msg.name(), MSG_TAB_UPDATE);
        assert!(msg.decode::<Vec<TabInfo>>().is_some());
    }

    #[test]
    fn factory_permission_result_roundtrip() {
        let msg = Message::permission_result(&PermissionStatus::Granted).unwrap();
        assert_eq!(msg.name(), MSG_PERMISSION_RESULT);
        assert_eq!(msg.decode::<PermissionStatus>(), Some(PermissionStatus::Granted));
    }

    #[test]
    fn message_name_constants() {
        assert_eq!(MSG_PANE_UPDATE, "pane-update");
        assert_eq!(MSG_TAB_UPDATE, "tab-update");
        assert_eq!(MSG_PERMISSION_RESULT, "permission-result");
        assert_eq!(MSG_EXECUTE_ACTION, "execute-action");
        assert_eq!(MSG_FOCUS_PANE, "do-focus-pane");
        assert_eq!(MSG_UPDATE_STATUS, "update-status");
        assert_eq!(ACTION_NEXT_SWAP_LAYOUT, "next-swap-layout");
        assert_eq!(ACTION_CLOSE_SELF, "close-self");
        assert_eq!(ACTION_HIDE_SELF, "hide-self");
    }
}
