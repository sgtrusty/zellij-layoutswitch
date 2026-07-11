use crate::output_port::{RecordingPort, set_port, clear_port};
use crate::worker::LayoutWorker;
use crate::message::{Message, MSG_EXECUTE_ACTION, MSG_FOCUS_PANE, ACTION_NEXT_SWAP_LAYOUT, ACTION_CLOSE_SELF};
use zellij_tile::prelude::*;

fn install() -> RecordingPort {
    let port = RecordingPort::default();
    set_port(Box::new(port.clone()));
    port
}

fn make_tab(name: &str, active: bool, layout: Option<&str>) -> TabInfo {
    TabInfo {
        name: name.to_string(),
        active,
        active_swap_layout_name: layout.map(String::from),
        ..Default::default()
    }
}

// ── focus-layout ────────────────────────────────────────────────────

#[test]
fn focus_layout_starts_discovery_when_cycle_unknown() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.on_message("focus-layout".into(), "target".into());

    assert!(worker.processing_layout);
    assert_eq!(worker.target_layout.as_deref(), Some("target"));

    let msgs = port.take_plugin_messages();
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].name, MSG_EXECUTE_ACTION);
    assert_eq!(msgs[0].payload, ACTION_NEXT_SWAP_LAYOUT);

    // update_status also fires
    assert!(msgs[1].payload.contains("target"));
    clear_port();
}

#[test]
fn focus_layout_rejected_when_already_processing() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.processing_layout = true;

    worker.on_message("focus-layout".into(), "target".into());
    assert!(port.take_plugin_messages().is_empty());
    clear_port();
}

#[test]
fn focus_layout_known_cycle_fires_distance_switches() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.layout_cycle = vec!["A".into(), "B".into(), "C".into()];
    worker.cycle_complete = true;
    worker.last_tab_infos = Some(vec![make_tab("t", true, Some("A"))]);

    worker.on_message("focus-layout".into(), "B".into());

    let msgs = port.take_plugin_messages();
    // 1 switch (distance A→B = 1) + 1 update_status
    assert_eq!(msgs[0].payload, ACTION_NEXT_SWAP_LAYOUT);
    assert!(worker.switches_fired);
    clear_port();
}

#[test]
fn focus_layout_known_cycle_already_at_target() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.layout_cycle = vec!["A".into(), "B".into()];
    worker.cycle_complete = true;
    worker.last_tab_infos = Some(vec![make_tab("t", true, Some("A"))]);

    worker.on_message("focus-layout".into(), "A".into());

    assert!(!worker.processing_layout);
    let msgs = port.take_plugin_messages();
    assert!(msgs.iter().all(|m| m.payload != ACTION_NEXT_SWAP_LAYOUT));
    clear_port();
}

#[test]
fn focus_layout_known_cycle_no_tab_info_fires_one() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.layout_cycle = vec!["A".into(), "B".into()];
    worker.cycle_complete = true;

    worker.on_message("focus-layout".into(), "B".into());

    let msgs = port.take_plugin_messages();
    assert_eq!(msgs[0].payload, ACTION_NEXT_SWAP_LAYOUT);
    clear_port();
}

// ── focus-pane ──────────────────────────────────────────────────────

#[test]
fn focus_pane_sets_state_and_tries_cache() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.on_message("focus-pane".into(), "My Terminal".into());

    assert!(worker.processing_pane);
    assert_eq!(worker.target_pane_title.as_deref(), Some("My Terminal"));
    // No cache — no focus message
    let msgs = port.take_plugin_messages();
    assert!(msgs.iter().all(|m| m.name != MSG_FOCUS_PANE));
    clear_port();
}

#[test]
fn focus_pane_rejected_when_already_processing() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.processing_pane = true;

    worker.on_message("focus-pane".into(), "My Terminal".into());
    assert!(port.take_plugin_messages().is_empty());
    clear_port();
}

#[test]
fn focus_pane_cache_hit_focuses_immediately() {
    let port = install();
    let mut worker = LayoutWorker::default();
    let mut panes_map = std::collections::HashMap::new();
    panes_map.insert(0, vec![
        zellij_tile::prelude::PaneInfo { id: 42, title: "MyTerm".into(), ..Default::default() },
    ]);
    worker.last_pane_manifest = Some(PaneManifest { panes: panes_map });

    worker.on_message("focus-pane".into(), "MyTerm".into());

    let msgs = port.take_plugin_messages();
    let focus = msgs.iter().find(|m| m.name == MSG_FOCUS_PANE).unwrap();
    assert_eq!(focus.payload, "42");
    assert!(!worker.processing_pane);
    clear_port();
}

#[test]
fn focus_pane_cache_miss_keeps_processing() {
    let port = install();
    let mut worker = LayoutWorker::default();
    let mut panes_map = std::collections::HashMap::new();
    panes_map.insert(0, vec![
        zellij_tile::prelude::PaneInfo { id: 1, title: "Other".into(), ..Default::default() },
    ]);
    worker.last_pane_manifest = Some(PaneManifest { panes: panes_map });

    worker.on_message("focus-pane".into(), "NonExistent".into());

    let msgs = port.take_plugin_messages();
    assert!(msgs.iter().all(|m| m.name != MSG_FOCUS_PANE));
    assert!(worker.processing_pane);
    clear_port();
}

// ── focus-stop ──────────────────────────────────────────────────────

#[test]
fn focus_stop_sends_close_self() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.on_message("focus-stop".into(), "".into());

    let msgs = port.take_plugin_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].name, MSG_EXECUTE_ACTION);
    assert_eq!(msgs[0].payload, ACTION_CLOSE_SELF);
    clear_port();
}

// ── dump-layouts ────────────────────────────────────────────────────

#[test]
fn dump_layouts_does_not_panic_with_empty_state() {
    let _port = install();
    let mut worker = LayoutWorker::default();
    worker.on_message("dump-layouts".into(), "".into());
    clear_port();
}

#[test]
fn dump_layouts_logs_with_populated_state() {
    let _port = install();
    let mut worker = LayoutWorker::default();
    worker.last_tab_infos = Some(vec![make_tab("tab1", true, Some("compact"))]);
    let mut panes_map = std::collections::HashMap::new();
    panes_map.insert(0, vec![
        zellij_tile::prelude::PaneInfo { id: 1, title: "Terminal 1".into(), ..Default::default() },
    ]);
    worker.last_pane_manifest = Some(PaneManifest { panes: panes_map });
    worker.layout_cycle = vec!["compact".into(), "expanded".into()];
    worker.cycle_complete = true;

    worker.on_message("dump-layouts".into(), "".into());
    clear_port();
}

// ── unknown command ─────────────────────────────────────────────────

#[test]
fn unknown_command_is_noop() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.on_message("unknown-command".into(), "data".into());
    assert!(port.take_plugin_messages().is_empty());
    clear_port();
}

// ── tab-update integration ──────────────────────────────────────────

#[test]
fn tab_update_discovery_records_new_layout() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("C".into());

    let msg = Message::tab_update(&[make_tab("t", true, Some("A"))]).unwrap();
    worker.on_message(msg.name().into(), msg.payload().into());

    assert!(worker.layout_cycle.contains(&"A".to_string()));
    assert_eq!(worker.visited_layouts, vec!["A".to_string()]);
    assert_eq!(worker.retry_count, 1);
    let msgs = port.take_plugin_messages();
    assert_eq!(msgs[0].payload, ACTION_NEXT_SWAP_LAYOUT);
    clear_port();
}

#[test]
fn tab_update_no_active_tab_early_return() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("B".into());

    let msg = Message::tab_update(&[make_tab("t", false, Some("A"))]).unwrap();
    worker.on_message(msg.name().into(), msg.payload().into());

    assert!(port.take_plugin_messages().is_empty());
    clear_port();
}

#[test]
fn tab_update_no_target_just_records() {
    let port = install();
    let mut worker = LayoutWorker::default();

    let msg = Message::tab_update(&[make_tab("t", true, Some("A"))]).unwrap();
    worker.on_message(msg.name().into(), msg.payload().into());

    assert!(worker.layout_cycle.contains(&"A".to_string()));
    assert!(port.take_plugin_messages().is_empty());
    clear_port();
}

// ── pane-update integration ─────────────────────────────────────────

#[test]
fn pane_update_with_target_focuses_matching_pane() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_pane_title = Some("MyTerm".into());

    let mut panes_map = std::collections::HashMap::new();
    panes_map.insert(0, vec![
        zellij_tile::prelude::PaneInfo { id: 7, title: "MyTerm".into(), ..Default::default() },
    ]);
    let manifest = PaneManifest { panes: panes_map };
    let msg = Message::pane_update(&manifest).unwrap();
    worker.on_message(msg.name().into(), msg.payload().into());

    let msgs = port.take_plugin_messages();
    let focus = msgs.iter().find(|m| m.name == MSG_FOCUS_PANE).unwrap();
    assert_eq!(focus.payload, "7");
    assert!(!worker.processing_pane);
    clear_port();
}

#[test]
fn pane_update_without_target_caches_manifest() {
    let port = install();
    let mut worker = LayoutWorker::default();

    let manifest = PaneManifest { panes: Default::default() };
    let msg = Message::pane_update(&manifest).unwrap();
    worker.on_message(msg.name().into(), msg.payload().into());

    assert!(worker.last_pane_manifest.is_some());
    assert!(port.take_plugin_messages().is_empty());
    clear_port();
}

// ── insta snapshots ─────────────────────────────────────────────────

#[test]
fn snapshot_idle_status_string() {
    let worker = LayoutWorker::default();
    let status = match (&worker.target_layout, &worker.target_pane_title) {
        (Some(layout), _) => format!("plugin-layoutswitch: switching to layout '{}'", layout),
        (_, Some(pane)) => format!("plugin-layoutswitch: focusing pane '{}'", pane),
        _ => "plugin-layoutswitch: IDLE".to_string(),
    };
    insta::assert_snapshot!("idle_status", status);
}

#[test]
fn snapshot_switching_status_string() {
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("compact".into());
    let status = match (&worker.target_layout, &worker.target_pane_title) {
        (Some(layout), _) => format!("plugin-layoutswitch: switching to layout '{}'", layout),
        (_, Some(pane)) => format!("plugin-layoutswitch: focusing pane '{}'", pane),
        _ => "plugin-layoutswitch: IDLE".to_string(),
    };
    insta::assert_snapshot!("switching_status", status);
}

#[test]
fn snapshot_focusing_status_string() {
    let mut worker = LayoutWorker::default();
    worker.target_pane_title = Some("Terminal 1".into());
    let status = match (&worker.target_layout, &worker.target_pane_title) {
        (Some(layout), _) => format!("plugin-layoutswitch: switching to layout '{}'", layout),
        (_, Some(pane)) => format!("plugin-layoutswitch: focusing pane '{}'", pane),
        _ => "plugin-layoutswitch: IDLE".to_string(),
    };
    insta::assert_snapshot!("focusing_status", status);
}

// ── permission-result integration ──────────────────────────────────────

#[test]
fn permission_result_forwards_to_worker() {
    let port = install();
    let mut worker = LayoutWorker::default();
    let msg = Message::permission_result(&PermissionStatus::Denied).unwrap();
    worker.on_message(msg.name().into(), msg.payload().into());

    assert!(!worker.processing_layout);
    let msgs = port.take_plugin_messages();
    assert!(msgs.is_empty());
    clear_port();
}
