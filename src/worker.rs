use zellij_tile::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{log, message::Message};
use crate::output_port::{output_port, OutputPort};

pub const LAYOUT_WORKER: &str = "layout";

#[doc(hidden)] pub const MAX_LAYOUT_RETRIES: usize = 20;

/// Background worker that manages layout switching and pane focus logic.
///
/// Caches the latest [`PaneManifest`] and [`TabInfo`] so it can resolve pane
/// titles to pane IDs and dump debug state on demand — without needing the
/// plugin to re-send stale data.
#[derive(Default, Serialize, Deserialize)]
pub struct LayoutWorker {
    #[doc(hidden)] pub target_layout: Option<String>,
    #[doc(hidden)] pub target_pane_title: Option<String>,
    #[doc(hidden)] pub visited_layouts: Vec<String>,
    #[doc(hidden)] pub retry_count: usize,
    #[doc(hidden)] pub processing_layout: bool,
    #[doc(hidden)] pub processing_pane: bool,
    #[doc(hidden)] pub last_pane_manifest: Option<PaneManifest>,
    #[doc(hidden)] pub last_tab_infos: Option<Vec<TabInfo>>,
    #[doc(hidden)] pub layout_cycle: Vec<String>,
    #[doc(hidden)] pub cycle_complete: bool,
    #[doc(hidden)] pub switches_fired: bool,
}

impl ZellijWorker<'_> for LayoutWorker {
    fn on_message(&mut self, name: String, payload: String) {
        let msg = Message::new(name, payload);

        match msg.name() {
            crate::message::MSG_PANE_UPDATE => {
                if let Some(pane_manifest) = msg.decode() {
                    self.handle_pane_update(pane_manifest);
                }
            }
            crate::message::MSG_TAB_UPDATE => {
                if let Some(tab_infos) = msg.decode() {
                    self.handle_tab_update(tab_infos);
                }
            }
            crate::message::MSG_PERMISSION_RESULT => {
                if let Some(result) = msg.decode() {
                    self.handle_permission_result(result);
                }
            }
            "focus-layout" => {
                if self.processing_layout {
                    log("Layout switch already in progress, ignoring");
                    return;
                }
                self.target_layout = Some(msg.payload().to_string());
                self.visited_layouts.clear();
                self.retry_count = 0;
                self.processing_layout = true;
                self.switches_fired = false;

                if self.is_cycle_known() {
                    if let Some(current) = self.get_current_layout() {
                        let distance = self.compute_distance(&current, msg.payload());
                        if distance == 0 {
                            self.reset_layout_process(&format!("LayoutSwitch: already at '{}'", msg.payload()));
                        } else {
                            log(format!(
                                "Cycle known ({} layouts): firing {} switches for '{}'",
                                self.layout_cycle.len(), distance, msg.payload()
                            ));
                            for _ in 0..distance {
                                output_port().post_to_plugin(Message::execute_action(crate::message::ACTION_NEXT_SWAP_LAYOUT).to_plugin());
                            }
                            self.switches_fired = true;
                        }
                    } else {
                        output_port().post_to_plugin(Message::execute_action(crate::message::ACTION_NEXT_SWAP_LAYOUT).to_plugin());
                    }
                } else {
                    log(format!("Layout cycle unknown, starting discovery for '{}'", msg.payload()));
                    output_port().post_to_plugin(Message::execute_action(crate::message::ACTION_NEXT_SWAP_LAYOUT).to_plugin());
                }
                self.update_status();
            }
            "focus-pane" => {
                if self.processing_pane {
                    log("Action already in progress, ignoring");
                    return;
                }
                self.processing_pane = true;
                self.target_pane_title = Some(msg.payload().to_string());
                self.try_focus_from_cache();
                self.update_status();
            }
            "dump-layouts" => {
                log("--- [DUMP] ALL PANES ---");
                if let Some(ref pane_manifest) = self.last_pane_manifest {
                    for (tab_index, panes) in &pane_manifest.panes {
                        for pane in panes {
                            log(format!("Tab: {} | Pane: '{}' | ID: {:?}", tab_index, pane.title, pane.id));
                        }
                    }
                }
                log("--- [DUMP] TABS ---");
                if let Some(ref tab_infos) = self.last_tab_infos {
                    for tab in tab_infos {
                        let layout = tab.active_swap_layout_name.as_deref().unwrap_or("BASE");
                        let marker = if tab.active { " [ACTIVE]" } else { "" };
                        log(format!("Tab: {}{} | Layout: {}", tab.name, marker, layout));
                    }
                }
                log("--- [DUMP] LAYOUT CYCLE ---");
                log(format!("Cycle complete: {}, Layouts: {:?}", self.cycle_complete, self.layout_cycle));
                self.update_status();
            }
            "focus-stop" => {
                output_port().post_to_plugin(Message::execute_action(crate::message::ACTION_CLOSE_SELF).to_plugin());
            }
            _ => ()
        }
    }
}
