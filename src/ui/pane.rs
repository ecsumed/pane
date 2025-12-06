use crate::command::Command;
use crate::mode::AppMode;
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::App;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Backend, Frame, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Widget};
use ratatui::Terminal;
use std::collections::HashMap;
use std::io;

pub fn draw_panes(
    frame: &mut Frame,
    manager: &PaneManager,
    commands: &HashMap<PaneKey, Command>,
) {
    fn draw_pane_node_recursive(
        frame: &mut Frame,
        area: Rect,
        manager: &PaneManager,
        commands: &HashMap<PaneKey, Command>,
        node_key: PaneKey,
    ) {
        let Some(node) = manager.nodes.get(node_key) else { return; };

        match &node.data {
            PaneNodeData::Single {} => {
                let is_active = node_key == manager.active_pane_id;
                
                let command = commands.get(&node_key);
                let (output, exec, interval_secs, state) = command
                    .map(|cmd| {
                        (
                            cmd.last_output.clone(),
                            cmd.exec.clone(),
                            cmd.interval.as_secs().to_string(),
                            cmd.state.to_string(),
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            "N/A".to_string(),
                            "N/A".to_string(),
                            "0".to_string(),
                            "N/A".to_string(),
                        )
                    });

                let border_style = if is_active {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let title_left = Line::from(vec![
                    " Command ".into(),
                    exec.blue().bold(),
                    " ".into(),
                    " Interval ".into(),
                    interval_secs.blue().bold(),
                    "s".blue().bold(),
                ]);

                let title_right = Line::from(vec![" State ".into(), state.blue().bold()]);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(title_left)
                    .title(title_right.right_aligned())
                    .border_style(border_style);
                let content_widget = Paragraph::new(output).block(block);

                frame.render_widget(content_widget, area);
            }
            PaneNodeData::Split { direction, children } => {
                let total_weight: u32 = children.iter().filter_map(|key| manager.nodes.get(*key)).map(|node| node.weight as u32).sum();
                
                let constraints = children.iter().map(|key| {
                    let weight = manager.nodes.get(*key).map_or(0, |node| node.weight);
                    Constraint::Ratio(weight as u32, total_weight)
                }).collect::<Vec<_>>();

                let chunks = Layout::default()
                    .direction(*direction)
                    .constraints(constraints)
                    .split(area);
                    
                for (chunk, child_key) in chunks.iter().zip(children.iter()) {
                    draw_pane_node_recursive( frame, *chunk, manager, commands, *child_key);
                }
            }
        }
    }
    
    let root_key = manager.nodes.iter().find(|(_, node)| node.parent.is_none()).map(|(key, _)| key);
    if let Some(key) = root_key {
        draw_pane_node_recursive(frame, frame.area(), manager, commands, key);
    } else {
        // TODO: Handle case where no root exists (e.g., draw a placeholder)
    }
}