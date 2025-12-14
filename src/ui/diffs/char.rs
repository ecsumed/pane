use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use similar::{ChangeTag, TextDiff};

pub fn render<'a>(current: &'a str, previous: &'a str, _query: &str) -> Vec<Line<'a>> {
    let diff = TextDiff::from_chars(previous, current);
    let mut lines = Vec::new();
    let mut current_line_spans = Vec::new();

    for change in diff.iter_all_changes() {
        let style = match change.tag() {
            ChangeTag::Delete => Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::CROSSED_OUT),
            ChangeTag::Insert => Style::default()
                .fg(Color::Green)
                .bg(Color::Rgb(0, 50, 0))
                .add_modifier(Modifier::UNDERLINED),
            ChangeTag::Equal => Style::default().fg(Color::DarkGray),
        };


        let value = change.value();

        if value.contains('\n') {
            let parts: Vec<&str> = value.split('\n').collect();
            
            for (i, part) in parts.iter().enumerate() {
                if !part.is_empty() {
                    current_line_spans.push(Span::styled(part.to_string(), style));
                }

                if i < parts.len() - 1 {
                    lines.push(Line::from(current_line_spans));
                    current_line_spans = Vec::new();
                }
            }
        } else {
            current_line_spans.push(Span::styled(value.to_string(), style));
        }    }

    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }

    lines
}