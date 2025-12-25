use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::command::Command;
use crate::config::AppConfig;
use crate::ui::diffs;
use crate::ui::DisplayType::{DiffChar, DiffWord, DiffLine};

pub fn render(frame: &mut Frame, area: Rect, config: &AppConfig, command: &Command) {
    let current_output = command.output_history.back();
    let previous_output = command.output_history.iter().rev().nth(1);

    let current_str = current_output.map(|c| c.output.as_str()).unwrap_or("");
    let previous_str = previous_output.map(|c| c.output.as_str()).unwrap_or("");
    let query = "";

    let lines = match command.display_type {
        DiffChar => diffs::char::render(current_str, previous_str, query),
        DiffWord => diffs::word::render(current_str, previous_str, query),
        DiffLine => diffs::line::render(current_str, previous_str, query),
        _ => Vec::new(),
    };


    let mut paragraph = Paragraph::new(Text::from(lines));

    if config.wrap {
        paragraph = paragraph.wrap(Wrap { trim: true });
    }

    frame.render_widget(paragraph, area);
}
