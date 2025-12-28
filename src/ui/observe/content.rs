use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use crate::command::Command;
use crate::config::AppConfig;
use crate::mode::DiffMode;
use crate::ui::diffs;

pub fn widget<'a>(
    config: &'a AppConfig,
    command: &'a Command,
    selected_idx: usize,
    diff_mode: DiffMode,
    search_query: &'a str,
) -> Paragraph<'a> {
    let p = &config.theme.palette;
    let current_len = command.output_history.len();
    
    // Logic to find which two history entries to compare
    let data_idx = current_len.saturating_sub(1).saturating_sub(selected_idx);
    let prev_data_idx = data_idx.checked_sub(1);

    let current_output = command.output_history.get(data_idx);
    let previous_output = prev_data_idx.and_then(|idx| command.output_history.get(idx));
    
    let current_text = current_output.map_or("", |c| &c.output);
    let previous_text = previous_output.map_or("", |c| &c.output);

    let display_text = diffs::render_diff(
        &config.theme,
        current_text,
        previous_text,
        diff_mode,
        search_query,
    );

    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(p.border_active)
        .padding(Padding::left(2));

    Paragraph::new(display_text).block(content_block)
}