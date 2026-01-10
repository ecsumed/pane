use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Frame, Rect};
use ratatui::widgets::Paragraph;

use crate::command::Command;
use crate::config::AppConfig;
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::ui::display_modes::render_command_output;
use crate::ui::panes::border::create_pane_block;
use crate::ui::panes::node_info::NodeInfo;
use crate::ui::utils::LayoutExt;

pub fn draw_recursive(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    manager: &PaneManager,
    commands: &HashMap<PaneKey, Command>,
    node_key: PaneKey,
) {
    let Some(node) = manager.nodes.get(node_key) else {
        return;
    };

    match &node.data {
        PaneNodeData::Single {} => {
            let is_active = node_key == manager.active_pane_id;

            let command = commands.get(&node_key);

            if let Some(cmd) = command {
                let node_info = NodeInfo::with_command(config, is_active, &cmd);

                let block = create_pane_block(config, node_info);

                render_command_output(frame, area, config, cmd, block);
            } else {
                let block = create_pane_block(config, NodeInfo::no_command(is_active));

                frame.render_widget(block.clone(), area);
                frame.render_widget(Paragraph::new("N/A"), block.inner(area));
            }
        }
        PaneNodeData::Split {
            direction,
            children,
        } => {
            let total_weight: u32 = children
                .iter()
                .filter_map(|key| manager.nodes.get(*key))
                .map(|node| node.weight as u32)
                .sum();

            let constraints = children
                .iter()
                .map(|key| {
                    let weight = manager.nodes.get(*key).map_or(0, |node| node.weight);
                    Constraint::Ratio(weight as u32, total_weight)
                })
                .collect::<Vec<_>>();

            let chunks = Layout::default()
                .direction(*direction)
                .constraints(constraints)
                .collapse_if(config.theme.collapse_borders && config.zen)
                .split(area);

            for (chunk, child_key) in chunks.iter().zip(children.iter()) {
                draw_recursive(frame, *chunk, config, manager, commands, *child_key);
            }
        }
    }
}
