use ratatui::{
    Frame, 
    layout::Rect,
    widgets::{Paragraph, Widget},
    text::{Line, Text},
    layout::{Alignment, Constraint, Direction, Layout},
};
use crate::command::Command;

pub fn render(frame: &mut Frame, area: Rect, command: &Command) {
    let history_lines: Vec<Line> = command.output_history.iter()
        .map(|s| Line::from(s.as_str()))
        .collect();
    
    let total_lines = history_lines.len();
    let display_height = area.height as usize;

    let lines_to_display = if total_lines > display_height {
        &history_lines[total_lines - display_height..]
    } else {
        &history_lines
    };
    
    let text_content = Text::from(Vec::from(lines_to_display));
    
    let widget = Paragraph::new(text_content);
    
    frame.render_widget(widget, area);
}
