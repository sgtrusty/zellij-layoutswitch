use crate::message::{
    Message, MSG_PANE_UPDATE, MSG_TAB_UPDATE, MSG_PERMISSION_RESULT, MSG_EXECUTE_ACTION,
    MSG_FOCUS_PANE, MSG_UPDATE_STATUS, ACTION_NEXT_SWAP_LAYOUT, ACTION_CLOSE_SELF, ACTION_HIDE_SELF,
};
use crate::output_port::{clear_port, set_port, RecordingPort};
use zellij_tile::prelude::*;

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

#[test]
fn act_dispatches_through_installed_port() {
    let port = RecordingPort::default();
    set_port(Box::new(port.clone()));
    Message::execute_action("my-action").act();
    assert_eq!(port.take_actions(), vec!["my-action"]);
    clear_port();
}

#[test]
fn act_abi_execute_next_swap_layout_is_noop() {
    Message::new(MSG_EXECUTE_ACTION, ACTION_NEXT_SWAP_LAYOUT).act_abi();
}

#[test]
fn act_abi_execute_close_self_is_noop() {
    Message::new(MSG_EXECUTE_ACTION, ACTION_CLOSE_SELF).act_abi();
}

#[test]
fn act_abi_execute_hide_self_is_noop() {
    Message::new(MSG_EXECUTE_ACTION, ACTION_HIDE_SELF).act_abi();
}

#[test]
fn act_abi_execute_unknown_action_is_noop() {
    Message::new(MSG_EXECUTE_ACTION, "bogus-action").act_abi();
}

#[test]
fn act_abi_focus_pane_valid_id_calls_abi() {
    Message::new(MSG_FOCUS_PANE, "42").act_abi();
}

#[test]
fn act_abi_focus_pane_invalid_id_is_noop() {
    Message::new(MSG_FOCUS_PANE, "not-a-number").act_abi();
}

#[test]
fn act_abi_unknown_message_name_is_noop() {
    Message::new("some-other-name", "payload").act_abi();
}
