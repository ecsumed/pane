use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget};
use ratatui::Frame;
use tui_input::Input;

use crate::config::theme::Palette;

const CHROME_HEIGHT: u16 = 4;
const MIN_PREVIEW_WIDTH: u16 = 30;

pub struct PickerAreas {
    pub list: Rect,
    pub preview: Option<Rect>,
    pub footer: Rect,
}

pub fn popup_area(frame_area: Rect, width: u16, content_rows: u16) -> Rect {
    let width = width
        .min(frame_area.width.saturating_sub(4))
        .max(frame_area.width.min(20));
    let height = (content_rows.max(1) + CHROME_HEIGHT).min(frame_area.height);
    Rect::new(
        frame_area.x + frame_area.width.saturating_sub(width) / 2,
        frame_area.y + frame_area.height.saturating_sub(height) / 2,
        width,
        height,
    )
    .intersection(frame_area)
}

pub fn draw_frame(
    frame: &mut Frame,
    area: Rect,
    p: &Palette,
    title: &str,
    filter: &Input,
    list_width: Option<u16>,
) -> PickerAreas {
    Clear.render(area, frame.buffer_mut());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(p.border_active)
        .title(Span::styled(format!(" {title} "), p.h1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [filter_area, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(inner);

    let filter_line = Line::from(vec![
        Span::styled(" / ", p.meta_highlight),
        Span::styled(" ", Style::default()),
        Span::styled(filter.value().to_string(), p.meta_value),
    ]);
    frame.render_widget(Paragraph::new(filter_line), filter_area);
    if filter_area.width > 0 && filter_area.height > 0 {
        let cursor_x = (filter_area.x + 4 + filter.visual_cursor() as u16)
            .min(filter_area.right().saturating_sub(1));
        frame.set_cursor_position((cursor_x, filter_area.y));
    }

    let (list, preview) = match list_width {
        Some(width) if body.width >= width + MIN_PREVIEW_WIDTH => {
            let [list, preview] =
                Layout::horizontal([Constraint::Length(width), Constraint::Min(0)]).areas(body);
            (list, Some(preview))
        }
        _ => (body, None),
    };

    PickerAreas {
        list,
        preview,
        footer,
    }
}

pub fn draw_list(
    frame: &mut Frame,
    area: Rect,
    p: &Palette,
    rows: Vec<ListItem>,
    selected_row: Option<usize>,
    empty_message: &str,
) {
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(format!(" {empty_message}"), p.meta_label)),
            area,
        );
        return;
    }

    let list = List::new(rows)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    let mut state = ListState::default().with_selected(selected_row);
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn draw_footer(frame: &mut Frame, area: Rect, line: Line) {
    frame.render_widget(Paragraph::new(line), area);
}

pub fn hints<'a>(p: &Palette, hints: &[(&'a str, &'a str)]) -> Line<'a> {
    let mut spans = vec![Span::raw(" ")];
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", p.meta_label));
        }
        spans.push(Span::styled(*key, p.meta_highlight));
        spans.push(Span::styled(format!(" {action}"), p.meta_label));
    }
    Line::from(spans)
}
