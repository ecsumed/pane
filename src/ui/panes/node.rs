use ratatui::prelude::{Frame, Rect};
use ratatui::widgets::Paragraph;

use crate::command::Command;
use crate::config::AppConfig;
use crate::ui::display_modes::render_command_output;
use crate::ui::panes::border::create_pane_block;
use crate::ui::panes::node_info::NodeInfo;

pub fn draw_pane(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    is_active: bool,
    command: Option<&Command>,
) {
    match command {
        Some(cmd) => {
            let block = create_pane_block(config, NodeInfo::with_command(config, is_active, cmd));
            render_command_output(frame, area, config, cmd, cmd.display_type, block);
        }
        None => {
            let block = create_pane_block(config, NodeInfo::no_command(is_active));
            frame.render_widget(block.clone(), area);
            frame.render_widget(Paragraph::new("N/A"), block.inner(area));
        }
    }
}
