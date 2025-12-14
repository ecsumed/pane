use std::collections::HashMap;

use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Frame, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::command::Command;
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::ui::display_modes::render_command_output;
use crate::ui::DisplayType;

fn create_pane_block<'a>(
    is_active: bool,
    exec_str: &'a str,
    interval_secs_str: &'a str,
    last_exec_time: &'a str,
    state_str: &'a str,
    display_type_str: &'a str,
) -> Block<'a> {
    let border_style = if is_active {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let title_top_left = Line::from(vec![
        "Every ".into(),
        interval_secs_str.dark_gray().bold(),
        "s: ".dark_gray().bold(),
        exec_str.blue().bold(),
    ]);
    let title_top_right = Line::from(vec![" State ".into(), state_str.blue().bold()]).right_aligned();

    let title_bottom_left = Line::from(vec![" Last updated: ".into(), last_exec_time.blue().bold()]);
    let title_bottom_right = Line::from(display_type_str).right_aligned();

    Block::default()
        .borders(Borders::ALL)
        .title(title_top_left)
        .title(title_top_right)
        .title_bottom(title_bottom_left)
        .title_bottom(title_bottom_right)
        .border_style(border_style)
        .padding(Padding::new(1, 1, 1, 1))
}

pub fn draw_panes(frame: &mut Frame, area: Rect, manager: &PaneManager, commands: &HashMap<PaneKey, Command>) {
    fn draw_pane_node_recursive(
        frame: &mut Frame,
        area: Rect,
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
                    let interval_str = cmd.interval.as_secs().to_string();
                    let state_str = cmd.state.to_string();
                    let display_str = format!("{:?}", cmd.display_type);
                    let last_exec_time = cmd.last_output()
                    .map(|c| c.time.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                    let block = create_pane_block(
                        is_active,
                        &cmd.exec,
                        &interval_str,
                        &last_exec_time,
                        &state_str,
                        &display_str,
                    );

                    render_command_output(frame, area, cmd, block);
                } else {
                    let display_str_na = format!("{:?}", DisplayType::RawText);

                    let block = create_pane_block(is_active, "N/A", "N/A", "N/A", "N/A", &display_str_na);

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
                    .split(area);

                for (chunk, child_key) in chunks.iter().zip(children.iter()) {
                    draw_pane_node_recursive(frame, *chunk, manager, commands, *child_key);
                }
            }
        }
    }

    let root_key = manager
        .nodes
        .iter()
        .find(|(_, node)| node.parent.is_none())
        .map(|(key, _)| key);
    if let Some(key) = root_key {
        draw_pane_node_recursive(frame, area, manager, commands, key);
    }
}
