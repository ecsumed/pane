use ratatui::text::Line;

use crate::{config::theme::Theme, ui::utils::highlight_query};

pub fn render<'a>(theme: &Theme, current: &'a str, query: &str) -> Vec<Line<'a>> {
    let p = &theme.palette;

    current
        .lines()
        .map(|line_content| {
            let spans = highlight_query(line_content, &query, p.output, p.search_match);
            Line::from(spans)
        })
        .collect()
}
