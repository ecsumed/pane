use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::command::Command;
use crate::config::theme::Theme;
use crate::config::AppConfig;
use crate::ui::diffs;
use crate::ui::DisplayType::{self, DiffChar, DiffLine, DiffWord};

fn compute(theme: &Theme, command: &Command, display_type: DisplayType) -> Vec<Line<'static>> {
    let mut latest = command.output_history.iter().rev();
    let current = latest
        .next()
        .map(|c| c.output.to_string())
        .unwrap_or_default();
    let previous = latest
        .next()
        .map(|c| c.output.to_string())
        .unwrap_or_default();

    diffs::owned(match display_type {
        DiffChar => diffs::char::render(theme, &current, &previous),
        DiffWord => diffs::word::render(theme, &current, &previous),
        DiffLine => diffs::line::render(theme, &current, &previous),
        _ => Vec::new(),
    })
}

pub fn refresh_view(theme: &Theme, command: &mut Command) {
    command.diff_view = matches!(command.display_type, DiffChar | DiffWord | DiffLine)
        .then(|| compute(theme, command, command.display_type));
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    command: &Command,
    display_type: DisplayType,
) {
    let lines = match &command.diff_view {
        Some(view) if display_type == command.display_type => diffs::borrowed(view),
        _ => compute(&config.theme, command, display_type),
    };

    let mut paragraph = Paragraph::new(Text::from(lines));

    if config.wrap {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    frame.render_widget(paragraph, area);
}
