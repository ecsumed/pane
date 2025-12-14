use similar::{ChangeTag, TextDiff};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn render<'a>(current: &'a str, query: &str) -> Vec<Line<'a>> {
    let query_lower = query.to_lowercase();
    let highlight_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    current
        .lines()
        .map(|line_content| {
            if query.is_empty() {
                return Line::from(line_content);
            }

            if let Some(index) = line_content.to_lowercase().find(&query_lower) {
                let start = index;
                let end = index + query.len();

                let prefix = &line_content[..start];
                let matched = &line_content[start..end];
                let suffix = &line_content[end..];

                Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(matched, highlight_style),
                    Span::raw(suffix),
                ])
            } else {
                Line::from(Span::styled(line_content, Style::default().fg(Color::DarkGray)))
            }
        })
        .collect()
}
