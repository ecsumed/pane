use ratatui::text::{Line, Span};
use similar::{ChangeTag, TextDiff};

use super::DIFF_TIMEOUT;

use crate::config::theme::Theme;

pub fn render<'a>(theme: &Theme, current: &'a str, previous: &'a str) -> Vec<Line<'a>> {
    let diff = TextDiff::configure()
        .timeout(DIFF_TIMEOUT)
        .diff_lines(previous, current);
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
            Span::styled(change.value().trim_end_matches('\n'), style),
        ]));
    }
    lines
}
