use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::config::AppConfig;
use crate::ui::utils::BlockExt;

pub fn widget<'a>(config: &'a AppConfig, value: &'a str) -> Paragraph<'a> {
    let prefix = Span::styled(" / ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));
    let search_text = Span::raw(value);
    let search_content = Line::from(vec![prefix, search_text]);

    let search_block = Block::default()
        .borders(Borders::ALL)
        .merge_if(config.theme.collapse_borders)
        .border_style(Style::default().fg(Color::DarkGray))
        .title_alignment(Alignment::Right)
        .title(Span::styled(" SEARCH ", Style::default().fg(Color::Yellow).bg(Color::Black)));

    Paragraph::new(search_content).block(search_block)
}