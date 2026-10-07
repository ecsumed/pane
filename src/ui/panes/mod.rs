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
