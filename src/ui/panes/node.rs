use std::collections::HashMap;

use humantime::format_duration;

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
use crate::ui::panes::border::create_pane_block;
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
                let interval_str = format!("{:?}", cmd.interval);
                let state_str = cmd.state.to_string();
                let display_str = format!("{:?}", cmd.display_type);
                let last_exec_time = cmd.last_output()
                    .map(|c| c.time.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                let duration = cmd.last_output()
                    .map(|c| format_duration(c.duration).to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                let block = create_pane_block(
                    config,
                    is_active,
                    &cmd.exec,
                    &interval_str,
                    &last_exec_time,
                    &duration,
                    &state_str,
                    &display_str,
                );

                render_command_output(frame, area, config, cmd, block);
            } else {
                let display_str_na = format!("{:?}", DisplayType::RawText);

                let block = create_pane_block(
                    config,
                    is_active, 
                    "N/A", 
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    &display_str_na
                );

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
