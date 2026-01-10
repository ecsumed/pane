use std::collections::HashMap;

use ratatui::prelude::{Frame, Rect};

use crate::command::Command;
use crate::config::AppConfig;
use crate::pane::{PaneKey, PaneManager};

mod border;
mod history_meter;
mod node;
mod node_info;

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    manager: &PaneManager,
    commands: &HashMap<PaneKey, Command>,
) {
    let root_key = manager
        .nodes
        .iter()
        .find(|(_, node)| node.parent.is_none())
        .map(|(key, _)| key);
    if let Some(key) = root_key {
        node::draw_recursive(frame, area, config, manager, commands, key);
    }
}
