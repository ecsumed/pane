use ratatui::text::Line;
use similar::{ChangeTag, TextDiff};

use crate::{config::theme::Theme, ui::utils::highlight_query};

pub fn render<'a>(
    theme: &Theme,
    current: &'a str,
    previous: &'a str,
    query: &str,
) -> Vec<Line<'a>> {
    let diff = TextDiff::from_words(previous, current);
    let mut lines = Vec::new();
    let mut current_line_spans = Vec::new();

    let p = &theme.palette;

    for change in diff.iter_all_changes() {
        let style = match change.tag() {
            ChangeTag::Delete => {
                if !theme.show_inline_deletions {
                    continue;
                }
                p.diff_remove
            }
            ChangeTag::Insert => p.diff_add,
            _ => p.output,
        };

        let value = change.value();

        if value.contains('\n') {
            let parts: Vec<&str> = value.split('\n').collect();

            for (i, part) in parts.iter().enumerate() {
                if !part.is_empty() {
                    let highlighted_parts = highlight_query(part, &query, style, p.search_match);
                    current_line_spans.extend(highlighted_parts);
                }

                if i < parts.len() - 1 {
                    lines.push(Line::from(current_line_spans));
                    current_line_spans = Vec::new();
                }
            }
        } else {
            let highlighted_parts = highlight_query(value, &query, style, p.search_match);
            current_line_spans.extend(highlighted_parts);
        }
    }

    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }

    lines
}
