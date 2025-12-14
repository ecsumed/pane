use similar::{ChangeTag, TextDiff};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

pub fn render<'a>(current: &'a str, previous: &'a str, _query: &str) -> Vec<Line<'a>> {
    let diff = TextDiff::from_lines(previous, current);
    let mut lines = Vec::new();

    for change in diff.iter_all_changes() {
        let (sign, style) = match change.tag() {
            ChangeTag::Delete => ("-", Style::default().fg(Color::Red)),
            ChangeTag::Insert => ("+", Style::default().fg(Color::Green)),
            ChangeTag::Equal => (" ", Style::default()),
        };
        
        lines.push(Line::from(vec![
            Span::styled(sign, style),
            Span::styled(change.value(), style),
        ]));
    }
    lines
}
