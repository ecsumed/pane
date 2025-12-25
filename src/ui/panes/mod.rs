use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Frame, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::command::Command;
use crate::config::AppConfig;
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::ui::display_modes::render_command_output;
use crate::ui::DisplayType;

mod border;
mod node;

pub fn draw(frame: &mut Frame, area: Rect, config: &AppConfig, manager: &PaneManager, commands: &HashMap<PaneKey, Command>) {
    let root_key = manager
        .nodes
        .iter()
        .find(|(_, node)| node.parent.is_none())
        .map(|(key, _)| key);
    if let Some(key) = root_key {
        node::draw_recursive(frame, area, config, manager, commands, key);
    }
}