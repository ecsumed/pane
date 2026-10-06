use ratatui::text::Line;

use crate::{config::theme::Theme, mode::DiffMode};

pub mod char;
pub mod line;
mod plain;
pub mod word;

pub fn render_diff<'a>(
    theme: &Theme,
    current: &'a str,
    previous: &'a str,
    mode: DiffMode,
) -> Vec<Line<'a>> {
    match mode {
        DiffMode::None => plain::render(theme, current),
        DiffMode::Line => line::render(theme, current, previous),
        DiffMode::Word => word::render(theme, current, previous),
        DiffMode::Char => char::render(theme, current, previous),
    }
}
