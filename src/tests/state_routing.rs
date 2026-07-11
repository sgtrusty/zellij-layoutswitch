use crate::output_port::{RecordingPort, set_port, clear_port};
use crate::State;
use crate::message::{MSG_PANE_UPDATE, MSG_TAB_UPDATE, MSG_PERMISSION_RESULT, MSG_EXECUTE_ACTION, MSG_UPDATE_STATUS, ACTION_NEXT_SWAP_LAYOUT};
use zellij_tile::prelude::*;

fn install() -> RecordingPort {
    let port = RecordingPort::default();
    set_port(Box::new(port.clone()));
    port
}

fn pipe_msg(name: &str, payload: Option<&str>) -> PipeMessage {
    PipeMessage {
        source: PipeSource::Cli(name.to_string()),
        name: name.to_string(),
        payload: payload.map(String::from),
        args: Default::default(),
        is_private: false,
    }
}

// ── pipe() routing ──────────────────────────────────────────────────

#[test]
fn pipe_focus_layout_forwarded_to_worker() {
    let port = install();
    let mut state = State::default();
    let result = state.pipe(pipe_msg("focus-layout", Some("compact")));
    assert!(!result);

    let msgs = port.take_worker_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].name, "focus-layout");
    assert_eq!(msgs[0].payload, "compact");
    clear_port();
}

#[test]
fn pipe_focus_pane_forwarded_to_worker() {
    let port = install();
    let mut state = State::default();
    let result = state.pipe(pipe_msg("focus-pane", Some("Terminal 1")));
    assert!(!result);

    let msgs = port.take_worker_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].name, "focus-pane");
    assert_eq!(msgs[0].payload, "Terminal 1");
    clear_port();
}

#[test]
fn pipe_dump_layouts_forwarded_to_worker() {
    let port = install();
    let mut state = State::default();
    let result = state.pipe(pipe_msg("dump-layouts", None));
    assert!(!result);

    let msgs = port.take_worker_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].name, "dump-layouts");
    assert_eq!(msgs[0].payload, "");
    clear_port();
}

#[test]
fn pipe_focus_stop_forwarded_to_worker() {
    let port = install();
    let mut state = State::default();
    let result = state.pipe(pipe_msg("focus-stop", None));
    assert!(!result);

    let msgs = port.take_worker_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].name, "focus-stop");
    clear_port();
}

#[test]
fn pipe_unknown_name_ignored() {
    let port = install();
    let mut state = State::default();
    let result = state.pipe(pipe_msg("unknown", Some("data")));
    assert!(!result);
    assert!(port.take_worker_messages().is_empty());
    clear_port();
}

#[test]
fn pipe_empty_payload_defaults_to_empty_string() {
    let port = install();
    let mut state = State::default();
    let result = state.pipe(pipe_msg("focus-layout", None));
    assert!(!result);

    let msgs = port.take_worker_messages();
    assert_eq!(msgs[0].payload, "");
    clear_port();
}

// ── update() routing ────────────────────────────────────────────────

#[test]
fn update_pane_update_forwards_to_worker() {
    let port = install();
    let mut state = State::default();
    let manifest = PaneManifest { panes: Default::default() };
    let should_render = state.update(Event::PaneUpdate(manifest));
    assert!(!should_render);

    let msgs = port.take_worker_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].name, MSG_PANE_UPDATE);
    clear_port();
}

#[test]
fn update_tab_update_forwards_to_worker() {
    let port = install();
    let mut state = State::default();
    let tabs = vec![TabInfo {
        name: "tab1".into(),
        active: true,
        ..Default::default()
    }];
    let should_render = state.update(Event::TabUpdate(tabs));
    assert!(!should_render);

    let msgs = port.take_worker_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].name, MSG_TAB_UPDATE);
    clear_port();
}

#[test]
fn update_permission_result_forwards_to_worker() {
    let port = install();
    let mut state = State::default();
    let should_render = state.update(Event::PermissionRequestResult(PermissionStatus::Granted));
    assert!(!should_render);

    let msgs = port.take_worker_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].name, MSG_PERMISSION_RESULT);
    clear_port();
}

#[test]
fn update_custom_message_status_sets_status_and_renders() {
    let _port = install();
    let mut state = State::default();
    let should_render = state.update(Event::CustomMessage(
        MSG_UPDATE_STATUS.into(),
        "plugin-layoutswitch: IDLE".into(),
    ));
    assert!(should_render);
    clear_port();
}

#[test]
fn update_custom_message_other_act_via_port() {
    let _port = install();
    let mut state = State::default();
    let should_render = state.update(Event::CustomMessage(
        MSG_EXECUTE_ACTION.into(),
        ACTION_NEXT_SWAP_LAYOUT.into(),
    ));
    assert!(!should_render);
    clear_port();
}

#[test]
fn update_unknown_event_no_render() {
    let _port = install();
    let mut state = State::default();
    let should_render = state.update(Event::Timer(1000.0));
    assert!(!should_render);
    clear_port();
}

// ── render() ────────────────────────────────────────────────────────

#[test]
fn render_prints_status_message() {
    let _port = install();
    let mut state = State::default();
    state.update(Event::CustomMessage(
        MSG_UPDATE_STATUS.into(),
        "plugin-layoutswitch: IDLE".into(),
    ));
    state.render(10, 80);
    clear_port();
}
