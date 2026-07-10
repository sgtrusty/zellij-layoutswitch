//! A Zellij plugin for switching between swap layouts and focusing panes by title.
//!
//! # Architecture
//! The plugin is split into two communicating parts:
//! - [`State`] runs on the plugin thread, handles Zellij events, and forwards
//!   commands to the worker.
//! - [`LayoutWorker`] runs in a background worker thread and processes pane/tab
//!   state, resolves pane titles to IDs, and cycles through swap layouts.

use zellij_tile::prelude::*;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

/// Plugin-side state. Receives events from Zellij and routes them to the worker.
#[derive(Default)]
pub struct State {
    status_message: String,
}

const MAX_LAYOUT_RETRIES: usize = 20;

/// Background worker that manages layout switching and pane focus logic.
///
/// Caches the latest [`PaneManifest`] and [`TabInfo`] so it can resolve pane
/// titles to pane IDs and dump debug state on demand — without needing the
/// plugin to re-send stale data.
#[derive(Default, Serialize, Deserialize)]
pub struct LayoutWorker {
    /// The swap-layout name to cycle toward (set by `focus-layout`).
    target_layout: Option<String>,
    /// The pane title to find and focus (set by `focus-pane`).
    target_pane_title: Option<String>,
    /// Layout names visited during the current cycle, used for cycle detection.
    visited_layouts: Vec<String>,
    /// How many `next-swap-layout` cycles have been sent for the current target.
    retry_count: usize,
    /// Whether a layout-switch action is in progress.
    processing_layout: bool,
    processing_pane: bool,
    /// Most recent pane manifest, cached so we can resolve titles immediately.
    last_pane_manifest: Option<PaneManifest>,
    /// Most recent tab info, cached so we can dump debug state immediately.
    last_tab_infos: Option<Vec<TabInfo>>,
}

pub const LAYOUT_WORKER: &str = "layout";

fn log(msg: impl std::fmt::Display) {
    eprintln!("[plugin-layoutswitch] {}", msg);
}

impl LayoutWorker {
    /// Process an incoming pane manifest.
    ///
    /// If a [`target_pane_title`](Self::target_pane_title) is pending, scans the
    /// manifest for a matching pane and sends [`do-focus-pane`] with its ID.
    fn handle_pane_update(&mut self, pane_manifest: PaneManifest) {
        self.last_pane_manifest = Some(pane_manifest.clone());
        if let Some(ref target_pane_title) = self.target_pane_title.clone() {
            let pane_id = self.find_pane_by_title(&pane_manifest, target_pane_title);
            if let Some(id) = pane_id {
                post_message_to_plugin(PluginMessage {
                    name: "do-focus-pane".to_string(),
                    payload: id.to_string(),
                    worker_name: None,
                });
                self.target_pane_title = None;
                self.processing_pane = false;
                self.update_status();
            }
        }
    }

    /// Scan the manifest for a pane whose title matches `target`.
    fn find_pane_by_title(&self, pane_manifest: &PaneManifest, target: &str) -> Option<u32> {
        for panes in pane_manifest.panes.values() {
            if let Some(pane) = panes.iter().find(|p| p.title.trim() == target.trim()) {
                return Some(pane.id);
            }
        }
        None
    }

    /// Process an incoming tab update.
    ///
    /// If a [`target_layout`](Self::target_layout) is pending, advances the
    /// swap-layout cycle toward it (with cycle detection).
    fn handle_tab_update(&mut self, tab_infos: Vec<TabInfo>) {
        self.last_tab_infos = Some(tab_infos.clone());
    
        // 1. Use an early return or 'if let' to avoid deep nesting
        let target_layout = match &self.target_layout {
            Some(layout) => layout,
            None => return,
        };
    
        let active_tab = match tab_infos.iter().find(|t| t.active) {
            Some(tab) => tab,
            None => return,
        };
    
        let current_layout = active_tab
            .active_swap_layout_name
            .as_deref()
            .unwrap_or("BASE");
    
        // 2. Success Condition
        if current_layout.eq_ignore_ascii_case(target_layout) {
            self.reset_layout_process(&format!("LayoutSwitch: reached '{}'", target_layout));
            return;
        }
    
        // 3. Cycle Detection
        if self.visited_layouts.contains(&current_layout.to_string()) {
            self.reset_layout_process(&format!("LayoutSwitch: '{}' not found (cycle detected)", target_layout));
            return;
        }
    
        // 4. Retry Logic
        self.visited_layouts.push(current_layout.to_string());
        self.retry_count += 1;
    
        if self.retry_count >= MAX_LAYOUT_RETRIES {
            self.reset_layout_process(&format!(
                "LayoutSwitch: '{}' not found after {} retries, giving up",
                target_layout, self.retry_count
            ));
        } else {
            post_message_to_plugin(PluginMessage {
                name: "execute-action".to_string(),
                payload: "next-swap-layout".to_string(),
                worker_name: None,
            });
        }
    }
    
    // 5. Helper method to reduce code duplication
    fn reset_layout_process(&mut self, message: &str) {
        log(message.to_string());
        self.target_layout = None;
        self.visited_layouts.clear();
        self.retry_count = 0;
        self.processing_layout = false;
        self.update_status();
    }

    fn handle_permission_result(&mut self, result: PermissionStatus) {
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

    /// If a [`target_pane_title`](Self::target_pane_title) is pending, try to
    /// resolve it immediately from the cached pane manifest.
    ///
    /// This avoids waiting for the next natural `pane-update` event.
    fn try_focus_from_cache(&mut self) {
        if let Some(ref target_pane_title) = self.target_pane_title.clone() {
            if let Some(ref pane_manifest) = self.last_pane_manifest {
                let pane_id = self.find_pane_by_title(pane_manifest, target_pane_title);
                if let Some(id) = pane_id {
                    post_message_to_plugin(PluginMessage {
                        name: "do-focus-pane".to_string(),
                        payload: id.to_string(),
                        worker_name: None,
                    });
                    self.target_pane_title = None;
                    self.processing_pane = false;
                }
            }
        }
    }

    /// Send the current status string to the plugin for display.
    fn update_status(&self) {
        let status = match (&self.target_layout, &self.target_pane_title) {
            (Some(layout), _) => format!("plugin-layoutswitch: switching to layout '{}'", layout),
            (_, Some(pane)) => format!("plugin-layoutswitch: focusing pane '{}'", pane),
            _ => "plugin-layoutswitch: IDLE".to_string(),
        };
        post_message_to_plugin(PluginMessage {
            name: "update-status".to_string(),
            payload: status,
            worker_name: None,
        });
    }
}

impl ZellijWorker<'_> for LayoutWorker {
    fn on_message(&mut self, message: String, payload: String) {
        match message.as_str() {
            "pane-update" => {
                if let Ok(pane_manifest) = serde_json::from_str::<PaneManifest>(&payload) {
                    self.handle_pane_update(pane_manifest);
                }
            }
            "tab-update" => {
                if let Ok(tab_infos) = serde_json::from_str::<Vec<TabInfo>>(&payload) {
                    self.handle_tab_update(tab_infos);
                }
            }
            "permission-result" => {
                if let Ok(result) = serde_json::from_str::<PermissionStatus>(&payload) {
                    self.handle_permission_result(result);
                }
            }
            "focus-layout" => {
                if self.processing_layout {
                    log("Layout switch already in progress, ignoring");
                    return;
                }
                self.target_layout = Some(payload);
                self.visited_layouts.clear();
                self.retry_count = 0;
                self.processing_layout = true;
                post_message_to_plugin(PluginMessage {
                    name: "execute-action".to_string(),
                    payload: "next-swap-layout".to_string(),
                    worker_name: None,
                });
                self.update_status();
            }
            "focus-pane" => {
                if self.processing_pane {
                    log("Action already in progress, ignoring");
                    return;
                }
                self.processing_pane = true;
                self.target_pane_title = Some(payload);
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
                self.update_status();
            }
            "focus-stop" => {
                post_message_to_plugin(PluginMessage {
                    name: "execute-action".to_string(),
                    payload: "close-self".to_string(),
                    worker_name: None,
                });
            }
            _ => ()
        }
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, _config: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ]);

        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::CustomMessage,
            EventType::PermissionRequestResult,
        ]);
    }

    /// Forward pipe commands (from Zellij keybindings or the CLI) to the worker.
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if matches!(pipe_message.name.as_str(), "focus-layout" | "focus-pane" | "dump-layouts" | "focus-stop") {
            let payload = pipe_message.payload.as_deref().unwrap_or("");
            let command_name = pipe_message.name;
            self.forward_to_worker(&command_name, payload);
            false
        } else {
            false
        }
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::CustomMessage(message_name, payload) => {
                match message_name.as_str() {
                    "update-status" => {
                        self.status_message = payload;
                        should_render = true;
                    }
                    "execute-action" => {
                        match payload.as_str() {
                            "next-swap-layout" => next_swap_layout(),
                            "close-self" => close_self(),
                            "hide-self" => hide_self(),
                            _ => ()
                        }
                    }
                    "do-focus-pane" => {
                        log(format!("Focusing pane ID: {}", payload));
                        if let Ok(id) = payload.parse::<u32>() {
                            focus_terminal_pane(id, false, false);
                        }
                    }
                    "focus-layout" | "focus-pane" | "dump-layouts" | "focus-stop" => {
                        self.forward_to_worker(&message_name, &payload);
                    }
                    _ => ()
                }
            }
            Event::PaneUpdate(pane_manifest) => {
                if let Ok(serialized_manifest) = serde_json::to_string(&pane_manifest) {
                    post_message_to(PluginMessage {
                        name: "pane-update".to_string(),
                        payload: serialized_manifest,
                        worker_name: Some(LAYOUT_WORKER.to_string()),
                    });
                }
            }
            Event::TabUpdate(tab_infos) => {
                if let Ok(serialized_infos) = serde_json::to_string(&tab_infos) {
                    post_message_to(PluginMessage {
                        name: "tab-update".to_string(),
                        payload: serialized_infos,
                        worker_name: Some(LAYOUT_WORKER.to_string()),
                    });
                }
            }
            Event::PermissionRequestResult(result) => {
                if let Ok(serialized_result) = serde_json::to_string(&result) {
                    post_message_to(PluginMessage {
                        name: "permission-result".to_string(),
                        payload: serialized_result,
                        worker_name: Some(LAYOUT_WORKER.to_string()),
                    });
                }
            }
            _ => (),
        }
        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        print!("{}", self.status_message);
    }
}

impl State {
    fn forward_to_worker(&mut self, command: &str, payload: &str) {
        post_message_to(PluginMessage {
            name: command.to_string(),
            payload: payload.to_string(),
            worker_name: Some(LAYOUT_WORKER.to_string()),
        });
    }
}
