use crate::config::AppConfig;
use crate::ui::utils::BlockExt;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn widget<'a>(config: &'a AppConfig, value: &'a str, is_focused: bool) -> Paragraph<'a> {
    let p = &config.theme.palette;

    let prefix = Span::styled(
        " / ",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    );
    let search_text = Span::raw(value);
    let search_content = Line::from(vec![prefix, search_text]);

    let border_style = if is_focused {
        p.border_active
    } else {
        p.border_inactive
    };

    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .merge_if(config.theme.collapse_borders)
        .title_alignment(Alignment::Right)
        .title(Span::styled(
            " SEARCH ",
            Style::default().fg(Color::Yellow).bg(Color::Black),
        ));

    Paragraph::new(search_content).block(search_block)
}
