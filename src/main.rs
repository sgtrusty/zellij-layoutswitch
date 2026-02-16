use zellij_tile::prelude::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    target_layout: Option<String>,
    target_pane_title: Option<String>,
    visited_layouts: Vec<String>,
    dump_requested: bool,
    processing_action: bool,
}

register_plugin!(State);

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

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "focus-layout" => {
                self.target_layout = pipe_message.payload.clone();
                self.visited_layouts.clear();
                self.processing_action = true;
                next_swap_layout();
                true
            },
            "focus-pane" => {
                self.target_pane_title = pipe_message.payload.clone();
                true
            },
            "dump-layouts" => {
                self.dump_requested = true;
                true
            },
            "focus-stop" => {
                close_self();
                true
            },
            _ => false
        }
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::CustomMessage(msg, payload) => {
                match msg.as_str() {
                    "focus-layout" => {
                        self.target_layout = Some(payload);
                        self.visited_layouts.clear();
                        self.processing_action = true;
                        next_swap_layout();
                        should_render = true;
                    },
                    "focus-pane" => {
                        self.target_pane_title = Some(payload);
                        should_render = true;
                    },
                    "dump-layouts" => {
                        self.dump_requested = true;
                        should_render = true;
                    },
                    "focus-stop" => {
                        close_self();
                    },
                    _ => ()
                }
            }
            Event::PaneUpdate(pane_manifest) => {
                if self.dump_requested {
                    eprintln!("--- [DUMP] ALL PANES ---");
                    for (tab_index, panes) in &pane_manifest.panes {
                        for pane in panes {
                            eprintln!("Tab: {} | Pane: '{}' | ID: {:?}", tab_index, pane.title, pane.id);
                        }
                    }
                    should_render = true;
                }

                if let Some(target_title) = self.target_pane_title.clone() {
                    let mut found_id = None;
                    for (_tab_index, panes) in &pane_manifest.panes {
                        if let Some(p) = panes.iter().find(|p| p.title.trim() == target_title.trim()) {
                            found_id = Some(p.id);
                            break;
                        }
                    }
                    if let Some(id) = found_id {
                        focus_terminal_pane(id, false);
                        self.target_pane_title = None;
                        should_render = true;
                    }
                }
            }
            Event::TabUpdate(tab_infos) => {
                if self.dump_requested {
                    eprintln!("--- [DUMP] TABS ---");
                    for tab in &tab_infos {
                        if tab.active {
                            eprintln!("Active Tab: {} | Layout: {:?}", tab.name, tab.active_swap_layout_name);
                        }
                    }
                    self.dump_requested = false;
                    should_render = true;
                }

                if let Some(target) = self.target_layout.clone() {
                    if let Some(tab) = tab_infos.iter().find(|t| t.active) {
                        let current_swap = tab.active_swap_layout_name
                            .clone()
                            .unwrap_or_else(|| "BASE".to_string());

                        if current_swap.eq_ignore_ascii_case(&target) {
                            self.target_layout = None;
                            self.visited_layouts.clear();
                            self.processing_action = false;
                        } 
                        else if self.visited_layouts.contains(&current_swap) {
                            eprintln!("[LayoutSwitch] Fail: '{}' not found.", target);
                            self.target_layout = None;
                            self.visited_layouts.clear();
                            self.processing_action = false;
                        } 
                        else {
                            self.visited_layouts.push(current_swap);
                            next_swap_layout();
                        }
                        should_render = true;
                    }
                }
            }
            Event::PermissionRequestResult(result) => {
                 if result == PermissionStatus::Granted {
                     hide_self();
                     should_render = true;
                 }
            }
            _ => (),
        }
        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        match (&self.target_layout, &self.target_pane_title) {
            (Some(l), _) => print!("Switching to Layout: {}", l),
            (_, Some(p)) => print!("Focusing Pane: {}", p),
            _ => print!("Layout Switcher: IDLE"),
        }
    }
}
