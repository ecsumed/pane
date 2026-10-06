use crate::config::AppConfig;
use crate::ui::picker::PickerStyle;
use crate::ui::utils::BlockExt;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn widget<'a>(config: &'a AppConfig, value: &'a str, is_focused: bool) -> Paragraph<'a> {
    let p = &config.theme.palette;
    let s = PickerStyle::from_palette(p);

    let prefix = Span::styled(" / ", s.accent);
    let search_text = Span::styled(value, s.text);
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
        .title(Span::styled(" Search ", s.muted));

    Paragraph::new(search_content).block(search_block)
}
