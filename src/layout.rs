use zellij_tile::prelude::*;

use crate::{log, message::Message};
use crate::output_port::{output_port, OutputPort};
use crate::worker::{LayoutWorker, MAX_LAYOUT_RETRIES};

impl LayoutWorker {
    pub(crate) fn handle_pane_update(&mut self, pane_manifest: PaneManifest) {
        self.last_pane_manifest = Some(pane_manifest.clone());
        if let Some(ref target_pane_title) = self.target_pane_title.clone() {
            let pane_id = self.find_pane_by_title(&pane_manifest, target_pane_title);
            if let Some(id) = pane_id {
                output_port().post_to_plugin(Message::focus_pane(id).to_plugin());
                self.target_pane_title = None;
                self.processing_pane = false;
                self.update_status();
            }
        }
    }

    pub(crate) fn find_pane_by_title(&self, pane_manifest: &PaneManifest, target: &str) -> Option<u32> {
        for panes in pane_manifest.panes.values() {
            if let Some(pane) = panes.iter().find(|p| p.title.trim() == target.trim()) {
                return Some(pane.id);
            }
        }
        None
    }

    pub(crate) fn record_layout(&mut self, layout: &str) {
        if !self.layout_cycle.iter().any(|l| l == layout) {
            self.layout_cycle.push(layout.to_string());
        }
    }

    pub(crate) fn is_cycle_known(&self) -> bool {
        self.cycle_complete
    }

    pub(crate) fn get_current_layout(&self) -> Option<String> {
        self.last_tab_infos
            .as_ref()
            .and_then(|tabs| tabs.iter().find(|t| t.active))
            .and_then(|t| t.active_swap_layout_name.clone())
    }

    pub(crate) fn compute_distance(&self, from: &str, to: &str) -> usize {
        let len = self.layout_cycle.len();
        if len == 0 {
            return 0;
        }
        let from_idx = self.layout_cycle.iter().position(|l| l == from).unwrap_or(0);
        let to_idx = self.layout_cycle.iter().position(|l| l == to).unwrap_or(0);
        (to_idx + len - from_idx) % len
    }

    pub(crate) fn handle_tab_update(&mut self, tab_infos: Vec<TabInfo>) {
        self.last_tab_infos = Some(tab_infos.clone());

        let active_tab = match tab_infos.iter().find(|t| t.active) {
            Some(tab) => tab,
            None => return,
        };

        let current_layout = active_tab
            .active_swap_layout_name
            .as_deref()
            .unwrap_or("BASE")
            .to_string();

        self.record_layout(&current_layout);

        let target_layout = match &self.target_layout {
            Some(layout) => layout.clone(),
            None => return,
        };

        if self.switches_fired {
            if current_layout.eq_ignore_ascii_case(&target_layout) {
                self.reset_layout_process(&format!("LayoutSwitch: reached '{}'", target_layout));
            } else {
                self.retry_count += 1;
                if self.retry_count >= MAX_LAYOUT_RETRIES {
                    self.reset_layout_process(&format!(
                        "LayoutSwitch: '{}' not reached after {} checks (direct mode)",
                        target_layout, self.retry_count
                    ));
                }
            }
            return;
        }

        if !self.cycle_complete {
            if self.visited_layouts.contains(&current_layout) {
                self.cycle_complete = true;
                log(format!("Layout cycle discovered: {:?}", self.layout_cycle));
                let distance = self.compute_distance(&current_layout, &target_layout);
                if distance == 0 {
                    self.reset_layout_process(&format!("LayoutSwitch: reached '{}'", target_layout));
                    output_port().post_to_plugin(Message::execute_action(crate::message::ACTION_NEXT_SWAP_LAYOUT).to_plugin());
                } else {
                    log(format!(
                        "Cycle discovered ({} layouts): firing {} switches to '{}'",
                        self.layout_cycle.len(), distance, target_layout
                    ));
                    for _ in 0..distance {
                        output_port().post_to_plugin(Message::execute_action(crate::message::ACTION_NEXT_SWAP_LAYOUT).to_plugin());
                    }
                    self.switches_fired = true;
                    self.retry_count = 0;
                }
            } else {
                self.visited_layouts.push(current_layout.clone());
                self.retry_count += 1;
                if self.retry_count >= MAX_LAYOUT_RETRIES {
                    self.reset_layout_process(&format!(
                        "LayoutSwitch: discovery failed after {} steps", self.retry_count
                    ));
                } else {
                    output_port().post_to_plugin(Message::execute_action(crate::message::ACTION_NEXT_SWAP_LAYOUT).to_plugin());
                }
            }
            return;
        }

        if current_layout.eq_ignore_ascii_case(&target_layout) {
            self.reset_layout_process(&format!("LayoutSwitch: reached '{}'", target_layout));
            return;
        }

        self.visited_layouts.push(current_layout.clone());
        self.retry_count += 1;

        if self.retry_count >= MAX_LAYOUT_RETRIES {
            self.reset_layout_process(&format!(
                "LayoutSwitch: '{}' not found after {} retries, giving up",
                target_layout, self.retry_count
            ));
        } else {
            output_port().post_to_plugin(Message::execute_action(crate::message::ACTION_NEXT_SWAP_LAYOUT).to_plugin());
        }
    }

    pub(crate) fn reset_layout_process(&mut self, message: &str) {
        log(message.to_string());
        self.target_layout = None;
        self.visited_layouts.clear();
        self.retry_count = 0;
        self.processing_layout = false;
        self.switches_fired = false;
        self.update_status();
    }

    pub(crate) fn handle_permission_result(&mut self, result: PermissionStatus) {
         match result {
             PermissionStatus::Granted => {
                 log("Permission granted");
             }
             PermissionStatus::Denied => {
                 log("Permission denied - plugin actions (layout switch, pane focus) will not work");
                 log("If loaded via load_permissions, pre-grant permissions in ~/.cache/zellij/permissions.kdl");
             }
         }
    }

    pub(crate) fn try_focus_from_cache(&mut self) {
        if let Some(ref target_pane_title) = self.target_pane_title.clone() {
            if let Some(ref pane_manifest) = self.last_pane_manifest {
                let pane_id = self.find_pane_by_title(pane_manifest, target_pane_title);
                if let Some(id) = pane_id {
                    output_port().post_to_plugin(Message::focus_pane(id).to_plugin());
                    self.target_pane_title = None;
                    self.processing_pane = false;
                }
            }
        }
    }

    pub(crate) fn update_status(&self) {
        let status = match (&self.target_layout, &self.target_pane_title) {
            (Some(layout), _) => format!("plugin-layoutswitch: switching to layout '{}'", layout),
            (_, Some(pane)) => format!("plugin-layoutswitch: focusing pane '{}'", pane),
            _ => "plugin-layoutswitch: IDLE".to_string(),
        };
        output_port().post_to_plugin(Message::update_status(status).to_plugin());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use proptest::prelude::*;

    use zellij_tile::prelude::PaneInfo;

    fn make_tab(name: &str, active: bool, layout: Option<&str>) -> TabInfo {
        TabInfo {
            name: name.to_string(),
            active,
            active_swap_layout_name: layout.map(String::from),
            ..Default::default()
        }
    }

    fn make_pane(id: u32, title: &str) -> PaneInfo {
        PaneInfo {
            id,
            title: title.to_string(),
            ..Default::default()
        }
    }

    // ── find_pane_by_title ──────────────────────────────────────────

    #[test]
    fn find_pane_found() {
        let worker = LayoutWorker::default();
        let mut panes = std::collections::HashMap::new();
        panes.insert(0, vec![
            make_pane(1, "Terminal 1"),
            make_pane(2, "Terminal 2"),
        ]);
        let manifest = PaneManifest { panes };
        assert_eq!(worker.find_pane_by_title(&manifest, "Terminal 2"), Some(2));
    }

    #[test]
    fn find_pane_not_found() {
        let worker = LayoutWorker::default();
        let manifest = PaneManifest { panes: Default::default() };
        assert_eq!(worker.find_pane_by_title(&manifest, "Missing"), None);
    }

    #[test]
    fn find_pane_trim_whitespace() {
        let worker = LayoutWorker::default();
        let mut panes = std::collections::HashMap::new();
        panes.insert(0, vec![
            make_pane(5, "  My Terminal  "),
        ]);
        let manifest = PaneManifest { panes };
        assert_eq!(worker.find_pane_by_title(&manifest, "My Terminal"), Some(5));
    }

    #[test]
    fn find_pane_multi_tab() {
        let worker = LayoutWorker::default();
        let mut panes = std::collections::HashMap::new();
        panes.insert(0, vec![
            make_pane(1, "A"),
        ]);
        panes.insert(1, vec![
            make_pane(2, "B"),
        ]);
        let manifest = PaneManifest { panes };
        assert_eq!(worker.find_pane_by_title(&manifest, "B"), Some(2));
    }

    // ── record_layout ───────────────────────────────────────────────

    #[test]
    fn record_layout_new() {
        let mut worker = LayoutWorker::default();
        worker.record_layout("A");
        assert_eq!(worker.layout_cycle, vec!["A"]);
    }

    #[test]
    fn record_layout_idempotent() {
        let mut worker = LayoutWorker::default();
        worker.record_layout("A");
        worker.record_layout("A");
        assert_eq!(worker.layout_cycle, vec!["A"]);
    }

    #[test]
    fn record_layout_preserves_order() {
        let mut worker = LayoutWorker::default();
        worker.record_layout("A");
        worker.record_layout("B");
        worker.record_layout("C");
        assert_eq!(worker.layout_cycle, vec!["A", "B", "C"]);
    }

    // ── compute_distance ────────────────────────────────────────────

    #[test]
    fn compute_distance_empty_cycle() {
        let worker = LayoutWorker::default();
        assert_eq!(worker.compute_distance("A", "B"), 0);
    }

    #[rstest]
    #[case("A", "B", 1)]
    #[case("B", "C", 1)]
    #[case("A", "C", 2)]
    #[case("C", "A", 1)]
    fn compute_distance_parameterized(#[case] from: &str, #[case] to: &str, #[case] expected: usize) {
        let mut worker = LayoutWorker::default();
        worker.layout_cycle = vec!["A".into(), "B".into(), "C".into()];
        assert_eq!(worker.compute_distance(from, to), expected);
    }

    #[test]
    fn compute_distance_unknown_layout() {
        let mut worker = LayoutWorker::default();
        worker.layout_cycle = vec!["A".into(), "B".into()];
        let dist = worker.compute_distance("X", "B");
        assert_eq!(dist, 1);
    }

    proptest! {
        #[test]
        fn compute_distance_always_less_than_cycle_len(a in 0usize..3, b in 0usize..3) {
            let layouts = vec!["A".to_string(), "B".to_string(), "C".to_string()];
            let mut worker = LayoutWorker::default();
            worker.layout_cycle = layouts.clone();
            let dist = worker.compute_distance(&layouts[a], &layouts[b]);
            prop_assert!(dist < layouts.len());
        }

        #[test]
        fn compute_distance_zero_when_same(idx in 0usize..3) {
            let layouts = vec!["A".to_string(), "B".to_string(), "C".to_string()];
            let mut worker = LayoutWorker::default();
            worker.layout_cycle = layouts.clone();
            prop_assert_eq!(worker.compute_distance(&layouts[idx], &layouts[idx]), 0);
        }

        #[test]
        fn record_layout_never_duplicates(entries in prop::collection::vec("[A-C]{1,5}", 0..10)) {
            let mut worker = LayoutWorker::default();
            for e in &entries {
                worker.record_layout(e);
            }
            let mut sorted = worker.layout_cycle.clone();
            sorted.dedup();
            prop_assert_eq!(worker.layout_cycle, sorted);
        }
    }

    // ── is_cycle_known ──────────────────────────────────────────────

    #[test]
    fn is_cycle_known_default_false() {
        let worker = LayoutWorker::default();
        assert!(!worker.is_cycle_known());
    }

    // ── get_current_layout ──────────────────────────────────────────

    #[test]
    fn get_current_layout_none_when_no_tabs() {
        let worker = LayoutWorker::default();
        assert_eq!(worker.get_current_layout(), None);
    }

    #[test]
    fn get_current_layout_returns_active() {
        let mut worker = LayoutWorker::default();
        worker.last_tab_infos = Some(vec![
            make_tab("tab1", false, Some("A")),
            make_tab("tab2", true, Some("B")),
        ]);
        assert_eq!(worker.get_current_layout(), Some("B".into()));
    }

    #[test]
    fn get_current_layout_none_when_no_active_swap() {
        let mut worker = LayoutWorker::default();
        worker.last_tab_infos = Some(vec![make_tab("tab1", true, None)]);
        assert_eq!(worker.get_current_layout(), None);
    }
}
