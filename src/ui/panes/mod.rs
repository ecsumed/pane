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
    config: &AppConfig,
    manager: &PaneManager,
    rects: &[(PaneKey, Rect)],
    commands: &HashMap<PaneKey, Command>,
) {
    for &(key, area) in rects {
        let is_active = key == manager.active_pane_id;
        node::draw_pane(frame, area, config, is_active, commands.get(&key));
    }
}

pub fn content_area(config: &AppConfig, command: &Command, pane: Rect) -> Rect {
    border::create_pane_block(
        config,
        node_info::NodeInfo::with_command(config, false, command),
    )
    .inner(pane)
}
