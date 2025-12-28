use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use similar::{ChangeTag, TextDiff};

use crate::config::theme::Theme;

pub fn render<'a>(theme: &Theme, current: &'a str, previous: &'a str, _query: &str) -> Vec<Line<'a>> {
    let diff = TextDiff::from_chars(previous, current);
    let mut lines = Vec::new();
    let mut current_line_spans = Vec::new();

    let p = &theme.palette;

    for change in diff.iter_all_changes() {
        let style = match change.tag() {
            ChangeTag::Delete => p.diff_remove,
            ChangeTag::Insert => p.diff_add,
            ChangeTag::Equal => p.output,
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