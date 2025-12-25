use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;
use crate::ui::DisplayType;

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command) {
    let history_lines: Vec<Line> = command
        .output_history
        .iter()
        .map(|entry| {
            let content = match command.display_type {
                DisplayType::MultiLineDateTime => {
                    format!("[{}] {}", entry.time.format("%Y-%m-%d %H:%M:%S"), entry.output)
                }
                DisplayType::MultiLineTime => {
                    format!("[{}] {}", entry.time.format("%H:%M:%S"), entry.output)
                }
                _ => entry.output.clone(),
            };
            Line::from(content)
        })
        .collect();

    let total_lines = history_lines.len();
    let display_height = area.height as usize;

    let lines_to_display = if total_lines > display_height {
        &history_lines[total_lines - display_height..]
    } else {
        &history_lines
    };

    let text_content = Text::from(Vec::from(lines_to_display));

    let mut widget = Paragraph::new(text_content);

    if config.wrap {
        widget = widget.wrap(ratatui::widgets::Wrap { trim: true });
    }

    frame.render_widget(widget, area);
}
