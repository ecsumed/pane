use similar::{ChangeTag, TextDiff};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::config::theme::Theme;

pub fn render<'a>(theme: &Theme, current: &'a str, previous: &'a str, _query: &str) -> Vec<Line<'a>> {
    let diff = TextDiff::from_lines(previous, current);
    let mut lines = Vec::new();

    let p = &theme.palette;

    for change in diff.iter_all_changes() {
        let (sign, style) = match change.tag() {
            ChangeTag::Delete => ("-", p.diff_remove),
            ChangeTag::Insert => ("+", p.diff_add),
            ChangeTag::Equal => (" ", p.output),
        };
        
        lines.push(Line::from(vec![
            Span::styled(sign, style),
            Span::styled(change.value(), style),
        ]));
    }
    lines
}
