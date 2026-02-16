use zellij_tile::prelude::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    plugin_id: Option<u32>,
    target_layout: Option<String>,
    editor_pane_id: Option<u32>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _config: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ]);
        
        // You need these subscriptions for the layout logic to trigger
        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::CustomMessage,
        ]);

        self.plugin_id = Some(get_plugin_ids().plugin_id);
    }
    
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        // In 0.43.1, pipe_message.name is a String, not an Option
        if pipe_message.name == "focus-layout" {
            self.target_layout = pipe_message.payload.clone();
            next_swap_layout();
        }
        false
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            // Logic for internal plugin messages
            Event::CustomMessage(msg, payload) => {
                if msg == "focus-layout" {
                    self.target_layout = Some(payload);
                    next_swap_layout();
                }
            }
            // Logic to find the ID of the pane named "Module Editor"
            Event::PaneUpdate(pane_manifest) => {
                for (_tab_index, panes) in pane_manifest.panes {
                    for pane in panes {
                        if pane.title == "Module Editor" {
                            self.editor_pane_id = Some(pane.id);
                        }
                    }
                }
            }
            // Logic to cycle layouts until the active one matches target_layout
            Event::TabUpdate(tab_infos) => {
                if let Some(ref target) = self.target_layout {
                    if let Some(tab) = tab_infos.iter().find(|t| t.active) {
                        let current_swap = tab.active_swap_layout_name.clone().unwrap_or_default();
                        
                        if current_swap == *target {
                            if let Some(id) = self.editor_pane_id {
                                focus_terminal_pane(id, false);
                                self.target_layout = None;
                            }
                        } else {
                            next_swap_layout();
                        }
                    }
                }
            }
            _ => (),
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        if let Some(ref target) = self.target_layout {
            print!("Checking layout... target is: {}", target);
        } else {
            print!("Layout Switcher Active (Idle)");
        }
    }
}
