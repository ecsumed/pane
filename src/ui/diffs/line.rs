use ratatui::text::{Line, Span};
use similar::{ChangeTag, TextDiff};

use crate::{config::theme::Theme, ui::utils::highlight_query};

pub fn render<'a>(
    theme: &Theme,
    current: &'a str,
    previous: &'a str,
    query: &str,
) -> Vec<Line<'a>> {
    let diff = TextDiff::from_lines(previous, current);
    let mut lines = Vec::new();

    let p = &theme.palette;

    for change in diff.iter_all_changes() {
        let (sign, style) = match change.tag() {
            ChangeTag::Delete => ("-", p.diff_remove),
            ChangeTag::Insert => ("+", p.diff_add),
            ChangeTag::Equal => (" ", p.output),
        };

        let mut spans = vec![Span::styled(sign, style)];
        let highlighted_parts = highlight_query(change.value(), &query, style, p.search_match);

        spans.extend(highlighted_parts);
        lines.push(Line::from(spans));
    }
    lines
}
