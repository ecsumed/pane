use ratatui::text::Line;

use crate::mode::DiffMode;

mod char;
mod line;
mod word;
mod plain;

pub fn render_diff<'a>(
    current: &'a str, 
    previous: &'a str, 
    mode: DiffMode,
    query: &str
) -> Vec<Line<'a>> {
    match mode {
        DiffMode::None => plain::render(current, query),
        DiffMode::Line => line::render(current, previous, query),
        DiffMode::Word => word::render(current, previous, query),
        DiffMode::Char => char::render(current, previous, query),
    }
}