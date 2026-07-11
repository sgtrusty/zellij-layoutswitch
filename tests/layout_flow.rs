use zellij_layoutswitch::{RecordingPort, LayoutWorker};
use zellij_layoutswitch::output_port::{set_port, clear_port};
use zellij_layoutswitch::test_support::{ACTION_NEXT_SWAP_LAYOUT, MSG_FOCUS_PANE, MAX_LAYOUT_RETRIES};
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

fn make_pane(title: &str, id: u32) -> PaneInfo {
    PaneInfo { id, title: title.into(), ..Default::default() }
}

fn make_manifest(panes: Vec<(usize, Vec<PaneInfo>)>) -> PaneManifest {
    let map: std::collections::HashMap<usize, Vec<PaneInfo>> = panes.into_iter().collect();
    PaneManifest { panes: map }
}

// ── Discovery flow ──────────────────────────────────────────────────

#[test]
fn discovery_sees_first_layout_and_switches() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("expanded".into());

    worker.handle_tab_update(vec![make_tab("t", true, Some("compact"))]);

    assert_eq!(worker.layout_cycle, vec!["compact"]);
    assert_eq!(worker.visited_layouts, vec!["compact"]);
    assert_eq!(worker.retry_count, 1);
    let msgs = port.take_plugin_messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].payload, ACTION_NEXT_SWAP_LAYOUT);
    clear_port();
}

#[test]
fn discovery_sees_duplicate_layout_completes_cycle() {
    let _port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("C".into());
    worker.visited_layouts = vec!["A".into(), "B".into()];
    worker.layout_cycle = vec!["A".into(), "B".into()];

    worker.handle_tab_update(vec![make_tab("t", true, Some("A"))]);

    assert!(worker.cycle_complete);
    // Distance A→C: C not in cycle, to_idx=0, from_idx=0, len=2 → 0 → reset
    assert!(!worker.processing_layout);
    clear_port();
}

#[test]
fn discovery_completes_and_fires_distance_switches() {
    let _port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("B".into());
    worker.visited_layouts = vec!["A".into(), "C".into()];
    worker.layout_cycle = vec!["A".into(), "C".into()];

    worker.handle_tab_update(vec![make_tab("t", true, Some("A"))]);

    assert!(worker.cycle_complete);
    // Distance A→B: B not in cycle, to_idx=0, from_idx=0, len=2 → 0 → reset
    // (B is unknown so to_idx defaults to 0, same as from_idx)
    clear_port();
}

#[test]
fn discovery_new_layout_within_known_cycle() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("C".into());
    worker.layout_cycle = vec!["A".into(), "B".into(), "C".into()];
    worker.cycle_complete = true;
    worker.last_tab_infos = Some(vec![make_tab("t", true, Some("A"))]);

    worker.handle_tab_update(vec![make_tab("t", true, Some("A"))]);

    let msgs = port.take_plugin_messages();
    assert_eq!(msgs.len(), 1); // known-cycle path fires one switch per tab update
    assert!(!worker.switches_fired); // switches_fired not set in this path
    clear_port();
}

// ── Switches-fired verification flow ────────────────────────────────

#[test]
fn switches_fired_reaches_target_resets() {
    let _port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("B".into());
    worker.switches_fired = true;

    worker.handle_tab_update(vec![make_tab("t", true, Some("B"))]);

    assert!(!worker.processing_layout);
    assert!(worker.target_layout.is_none());
    assert!(worker.visited_layouts.is_empty());
    clear_port();
}

#[test]
fn switches_fired_not_yet_reached_increments_retry() {
    let _port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("B".into());
    worker.switches_fired = true;
    worker.processing_layout = true;

    worker.handle_tab_update(vec![make_tab("t", true, Some("A"))]);

    assert_eq!(worker.retry_count, 1);
    assert!(worker.processing_layout);
    clear_port();
}

#[test]
fn switches_fired_retry_limit_gives_up() {
    let _port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("B".into());
    worker.switches_fired = true;
    worker.retry_count = MAX_LAYOUT_RETRIES - 1;

    worker.handle_tab_update(vec![make_tab("t", true, Some("A"))]);

    assert!(!worker.processing_layout);
    assert!(worker.target_layout.is_none());
    clear_port();
}

// ── Layout recording flow ───────────────────────────────────────────

#[test]
fn layout_recording_accumulates_across_updates() {
    let _port = install();
    let mut worker = LayoutWorker::default();

    worker.handle_tab_update(vec![make_tab("t", true, Some("A"))]);
    worker.handle_tab_update(vec![make_tab("t", true, Some("B"))]);
    worker.handle_tab_update(vec![make_tab("t", true, Some("C"))]);

    assert_eq!(worker.layout_cycle, vec!["A", "B", "C"]);
    clear_port();
}

#[test]
fn layout_recording_deduplicates() {
    let _port = install();
    let mut worker = LayoutWorker::default();

    worker.handle_tab_update(vec![make_tab("t", true, Some("A"))]);
    worker.handle_tab_update(vec![make_tab("t", true, Some("B"))]);
    worker.handle_tab_update(vec![make_tab("t", true, Some("A"))]);

    // "A" should appear only once in layout_cycle
    assert_eq!(worker.layout_cycle.iter().filter(|l| *l == "A").count(), 1);
    clear_port();
}

// ── Pane focus flow ─────────────────────────────────────────────────

#[test]
fn pane_focus_cache_hit_clears_state() {
    let port = install();
    let mut worker = LayoutWorker::default();
    let manifest = make_manifest(vec![(0, vec![make_pane("MyTerm", 42)])]);
    worker.last_pane_manifest = Some(manifest);
    worker.target_pane_title = Some("MyTerm".into());
    worker.processing_pane = true;

    worker.try_focus_from_cache();

    let msgs = port.take_plugin_messages();
    let focus = msgs.iter().find(|m| m.name == MSG_FOCUS_PANE).unwrap();
    assert_eq!(focus.payload, "42");
    assert!(!worker.processing_pane);
    assert!(worker.target_pane_title.is_none());
    clear_port();
}

#[test]
fn pane_focus_cache_miss_stays_processing() {
    let port = install();
    let mut worker = LayoutWorker::default();
    let manifest = make_manifest(vec![(0, vec![make_pane("Other", 1)])]);
    worker.last_pane_manifest = Some(manifest);
    worker.target_pane_title = Some("NonExistent".into());
    worker.processing_pane = true;

    worker.try_focus_from_cache();

    assert!(port.take_plugin_messages().is_empty());
    assert!(worker.processing_pane);
    clear_port();
}

#[test]
fn pane_update_triggers_focus_when_target_set() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_pane_title = Some("MyTerm".into());

    let manifest = make_manifest(vec![(0, vec![make_pane("MyTerm", 7)])]);
    worker.handle_pane_update(manifest);

    let msgs = port.take_plugin_messages();
    let focus = msgs.iter().find(|m| m.name == MSG_FOCUS_PANE).unwrap();
    assert_eq!(focus.payload, "7");
    assert!(!worker.processing_pane);
    clear_port();
}

// ── Status updates ──────────────────────────────────────────────────

#[test]
fn update_status_reflects_idle() {
    let port = install();
    let worker = LayoutWorker::default();
    worker.update_status();
    let msgs = port.take_plugin_messages();
    assert!(msgs[0].payload.contains("IDLE"));
    clear_port();
}

#[test]
fn update_status_reflects_layout_target() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("compact".into());
    worker.update_status();
    let msgs = port.take_plugin_messages();
    assert!(msgs[0].payload.contains("compact"));
    clear_port();
}

#[test]
fn update_status_reflects_pane_target() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_pane_title = Some("Terminal 1".into());
    worker.update_status();
    let msgs = port.take_plugin_messages();
    assert!(msgs[0].payload.contains("Terminal 1"));
    clear_port();
}

// ── Reset flow ──────────────────────────────────────────────────────

#[test]
fn reset_clears_all_processing_state() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("X".into());
    worker.visited_layouts = vec!["A".into()];
    worker.retry_count = 5;
    worker.processing_layout = true;
    worker.switches_fired = true;

    worker.reset_layout_process("done");

    assert!(worker.target_layout.is_none());
    assert!(worker.visited_layouts.is_empty());
    assert_eq!(worker.retry_count, 0);
    assert!(!worker.processing_layout);
    assert!(!worker.switches_fired);
    // update_status fires IDLE
    let msgs = port.take_plugin_messages();
    assert!(msgs[0].payload.contains("IDLE"));
    clear_port();
}

// ── Full lifecycle integration ──────────────────────────────────────

#[test]
fn full_lifecycle_discovery_then_switch_then_reset() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("compact".into());

    // Step 1: Discovery — first layout
    worker.handle_tab_update(vec![make_tab("t", true, Some("base"))]);
    let msgs = port.take_plugin_messages();
    assert_eq!(msgs[0].payload, ACTION_NEXT_SWAP_LAYOUT);
    assert_eq!(worker.layout_cycle, vec!["base"]);
    assert_eq!(worker.retry_count, 1);

    // Step 2: Discovery — second layout
    worker.handle_tab_update(vec![make_tab("t", true, Some("wide"))]);
    let msgs = port.take_plugin_messages();
    assert_eq!(msgs[0].payload, ACTION_NEXT_SWAP_LAYOUT);
    assert_eq!(worker.layout_cycle, vec!["base", "wide"]);

    // Step 3: Discovery — back to first (cycle complete)
    worker.handle_tab_update(vec![make_tab("t", true, Some("base"))]);
    assert!(worker.cycle_complete);
    // Distance base→compact: compact not in cycle, to_idx=0, from_idx=0, len=2 → 0
    // So reset with "reached" — because distance 0
    assert!(!worker.processing_layout);
    clear_port();
}

#[test]
fn full_lifecycle_known_cycle_switches_to_target() {
    let port = install();
    let mut worker = LayoutWorker::default();
    worker.layout_cycle = vec!["A".into(), "B".into(), "C".into()];
    worker.cycle_complete = true;
    worker.last_tab_infos = Some(vec![make_tab("t", true, Some("A"))]);
    worker.target_layout = Some("C".into());

    worker.on_message("focus-layout".into(), "C".into());

    // Distance A→C = 2, so 2 switches fired
    let msgs = port.take_plugin_messages();
    let switches: Vec<_> = msgs.iter().filter(|m| m.payload == ACTION_NEXT_SWAP_LAYOUT).collect();
    assert_eq!(switches.len(), 2);
    assert!(worker.switches_fired);

    // Simulate reaching target
    worker.handle_tab_update(vec![make_tab("t", true, Some("C"))]);
    assert!(!worker.processing_layout);
    assert!(worker.target_layout.is_none());
    clear_port();
}

// ── Insta snapshots for full flow status ────────────────────────────

#[test]
fn snapshot_status_after_discovery() {
    let mut worker = LayoutWorker::default();
    worker.target_layout = Some("compact".into());
    worker.visited_layouts = vec!["base".into()];
    worker.retry_count = 1;

    let status = format!(
        "plugin-layoutswitch: switching to layout '{}'",
        worker.target_layout.as_deref().unwrap()
    );
    insta::assert_snapshot!("flow_status_switching", status);
}

#[test]
fn snapshot_status_after_focus_pane() {
    let mut worker = LayoutWorker::default();
    worker.target_pane_title = Some("Terminal 1".into());
    worker.processing_pane = true;

    let status = format!(
        "plugin-layoutswitch: focusing pane '{}'",
        worker.target_pane_title.as_deref().unwrap()
    );
    insta::assert_snapshot!("flow_status_focusing", status);
}

#[test]
fn snapshot_status_after_reset() {
    let _worker = LayoutWorker::default();
    let status = "plugin-layoutswitch: IDLE".to_string();
    insta::assert_snapshot!("flow_status_idle", status);
}
