use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::command::Command;
use crate::config::theme::Theme;
use crate::config::AppConfig;
use crate::ui::diffs;
use crate::ui::DisplayType::{self, DiffChar, DiffLine, DiffWord};

fn compute<'a>(theme: &Theme, command: &'a Command, display_type: DisplayType) -> Vec<Line<'a>> {
    let mut latest = command.output_history.iter().rev();
    let current = latest.next().map_or("", |c| c.output.as_str());
    let previous = latest.next().map_or("", |c| c.output.as_str());

    match display_type {
        DiffChar => diffs::char::render(theme, current, previous),
        DiffWord => diffs::word::render(theme, current, previous),
        DiffLine => diffs::line::render(theme, current, previous),
        _ => Vec::new(),
    }
}

fn owned(lines: Vec<Line<'_>>) -> Vec<Line<'static>> {
    lines
        .into_iter()
        .map(|line| Line {
            style: line.style,
            alignment: line.alignment,
            spans: line
                .spans
                .into_iter()
                .map(|span| Span::styled(span.content.into_owned(), span.style))
                .collect(),
        })
        .collect()
}

fn borrowed<'a>(lines: &'a [Line<'static>]) -> Vec<Line<'a>> {
    lines
        .iter()
        .map(|line| Line {
            style: line.style,
            alignment: line.alignment,
            spans: line
                .spans
                .iter()
                .map(|span| Span::styled(span.content.as_ref(), span.style))
                .collect(),
        })
        .collect()
}

pub fn refresh_view(theme: &Theme, command: &mut Command) {
    command.diff_view = matches!(command.display_type, DiffChar | DiffWord | DiffLine)
        .then(|| owned(compute(theme, command, command.display_type)));
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    command: &Command,
    display_type: DisplayType,
) {
    let lines = match &command.diff_view {
        Some(view) if display_type == command.display_type => borrowed(view),
        _ => compute(&config.theme, command, display_type),
    };

    let mut paragraph = Paragraph::new(Text::from(lines));

    if config.wrap {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    frame.render_widget(paragraph, area);
}
