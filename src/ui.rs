use crate::command::Command;
use crate::mode::InputMode;
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::App;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Backend, Frame, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};
use ratatui::Terminal;
use std::collections::HashMap;
use std::io;

fn centered_rect(percent_x: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw_main_ui(app: &App, frame: &mut Frame) {
    draw_panes(
        &app.pane_manager,
        frame,
        frame.area(),
        &app.tasks,
    );
}

pub fn draw_input_popup(app: &App, frame: &mut Frame) {
    if app.input_state.mode == InputMode::Editing {
        let popup_area = centered_rect(60, frame.area());
        frame.render_widget(Clear, popup_area);
        let input_widget = Paragraph::new(app.input_state.input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Command"),
        );
        frame.render_widget(input_widget, popup_area);
    }
}

pub fn draw_panes(
    manager: &PaneManager,
    frame: &mut Frame,
    area: Rect,
    commands: &HashMap<PaneKey, Command>,
) {
    fn draw_pane_node_recursive(
        manager: &PaneManager,
        frame: &mut Frame,
        area: Rect,
        node_key: PaneKey,
        commands: &HashMap<PaneKey, Command>,
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
                    draw_pane_node_recursive(manager, frame, *chunk, *child_key, commands);
                }
            }
        }
    }
    
    let root_key = manager.nodes.iter().find(|(_, node)| node.parent.is_none()).map(|(key, _)| key);
    if let Some(key) = root_key {
        draw_pane_node_recursive(manager, frame, area, key, commands);
    } else {
        // TODO: Handle case where no root exists (e.g., draw a placeholder)
    }
}

pub fn manage_cursor<B: Backend>(app: &App, terminal: &mut Terminal<B>) -> io::Result<()> {
    match app.input_state.mode {
        InputMode::Editing => {
            let term_size = terminal.size()?;
            let term_rect = Rect::new(0, 0, term_size.width, term_size.height);
            let popup_area = centered_rect(60, term_rect);

            let cursor_x = popup_area.x + app.input_state.input.cursor() as u16 + 1;
            let cursor_y = popup_area.y + 1;

            terminal.set_cursor_position((cursor_x, cursor_y))?;
            terminal.show_cursor()
        }
        InputMode::Normal => terminal.hide_cursor(),
    }
}
