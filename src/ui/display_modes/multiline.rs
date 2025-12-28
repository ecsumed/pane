use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;
use crate::ui::DisplayType;

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command) {
    let p = &config.theme.palette;

    let history_lines: Vec<Line> = command
        .output_history
        .iter()
        .map(|entry| {
            let dt_string = match command.display_type {
                DisplayType::MultiLineDateTime => {
                    // Use .format() for Chrono types
                    format!("[{}]", entry.time.format("%Y-%m-%d %H:%M:%S"))
                }
                DisplayType::MultiLineTime => {
                    format!("[{}]", entry.time.format("%H:%M:%S"))
                }
                _ => String::new(),
            };

            Line::from(vec![
                Span::styled(dt_string, p.multiline_timestamp),
                Span::styled(format!(" {}", entry.output), p.output),
            ])
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
        widget = widget.wrap(Wrap { trim: true });
    }

    frame.render_widget(widget, area);
}
