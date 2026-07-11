use crate::message::{ACTION_NEXT_SWAP_LAYOUT, MSG_EXECUTE_ACTION, MSG_FOCUS_PANE, Message};
use crate::output_port::*;
use zellij_tile::prelude::*;

#[test]
fn recording_port_captures_post_to_plugin() {
    let port = RecordingPort::default();
    port.post_to_plugin(Message::execute_action(ACTION_NEXT_SWAP_LAYOUT).to_plugin());
    assert_eq!(port.plugin_messages.borrow().len(), 1);
    assert_eq!(port.plugin_messages.borrow()[0].name, MSG_EXECUTE_ACTION);
}

#[test]
fn recording_port_captures_post_to() {
    let port = RecordingPort::default();
    port.post_to(Message::new("test", "data").to_worker());
    assert_eq!(port.worker_messages.borrow().len(), 1);
}

#[test]
fn recording_port_captures_actions() {
    let port = RecordingPort::default();
    port.act(&Message::new(MSG_FOCUS_PANE, "42"));
    assert_eq!(*port.actions.borrow(), vec!["42"]);
}

#[test]
fn take_plugin_messages_drains() {
    let port = RecordingPort::default();
    port.post_to_plugin(Message::execute_action(ACTION_NEXT_SWAP_LAYOUT).to_plugin());
    assert_eq!(port.take_plugin_messages().len(), 1);
    assert!(port.take_plugin_messages().is_empty());
}

#[test]
fn take_worker_messages_drains() {
    let port = RecordingPort::default();
    port.post_to(Message::new("x", "y").to_worker());
    assert_eq!(port.take_worker_messages().len(), 1);
    assert!(port.take_worker_messages().is_empty());
}

#[test]
fn take_actions_drains() {
    let port = RecordingPort::default();
    port.act(&Message::new(MSG_FOCUS_PANE, "1"));
    assert_eq!(port.take_actions().len(), 1);
    assert!(port.take_actions().is_empty());
}

#[test]
fn set_and_clear_port() {
    let port = RecordingPort::default();
    set_port(Box::new(port));
    clear_port();
}

#[test]
fn default_port_falls_through_to_abi_when_no_port_set() {
    clear_port();
    let default = output_port();
    default.post_to_plugin(Message::execute_action(ACTION_NEXT_SWAP_LAYOUT).to_plugin());
    default.post_to(Message::new("x", "y").to_worker());
    default.act(&Message::new(MSG_FOCUS_PANE, "1"));
    clear_port();
}
