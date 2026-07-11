use zellij_tile::prelude::*;
use std::collections::BTreeMap;

use crate::message::Message;
use crate::output_port::{output_port, OutputPort};

/// Plugin-side state. Receives events from Zellij and routes them to the worker.
#[derive(Default)]
pub struct State {
    status_message: String,
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
                    crate::message::MSG_UPDATE_STATUS => {
                        self.status_message = payload;
                        should_render = true;
                    }
                    _ => Message::new(message_name, payload).act(),
                }
            }
            Event::PaneUpdate(pane_manifest) => {
                if let Some(msg) = Message::pane_update(&pane_manifest) {
                    output_port().post_to(msg.to_worker());
                }
            }
            Event::TabUpdate(tab_infos) => {
                if let Some(msg) = Message::tab_update(&tab_infos) {
                    output_port().post_to(msg.to_worker());
                }
            }
            Event::PermissionRequestResult(result) => {
                if let Some(msg) = Message::permission_result(&result) {
                    output_port().post_to(msg.to_worker());
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
        output_port().post_to(Message::new(command, payload).to_worker());
    }
}
