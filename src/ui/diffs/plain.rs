use ratatui::text::{Line, Span};

use crate::config::theme::Theme;

pub fn render<'a>(theme: &Theme, current: &'a str) -> Vec<Line<'a>> {
    current
        .lines()
        .map(|line| Line::from(Span::styled(line, theme.palette.output)))
        .collect()
}
