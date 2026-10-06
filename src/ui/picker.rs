use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget};
use ratatui::Frame;
use tui_input::Input;

use crate::config::theme::Palette;

const CHROME_HEIGHT: u16 = 4;
const MIN_PREVIEW_WIDTH: u16 = 30;

pub struct PickerStyle {
    pub border: Style,
    pub title: Style,
    pub text: Style,
    pub key: Style,
    pub muted: Style,
    pub disabled: Style,
    pub accent: Style,
    pub marker: Style,
    pub selected: Style,
    pub error: Style,
}

impl PickerStyle {
    pub fn from_palette(p: &Palette) -> Self {
        let muted = p.meta_label.remove_modifier(Modifier::BOLD);
        Self {
            border: p.meta_secondary,
            title: p.meta_value.add_modifier(Modifier::BOLD),
            text: p.meta_value,
            key: p.meta_value.add_modifier(Modifier::BOLD),
            muted,
            disabled: muted.add_modifier(Modifier::DIM),
            accent: p.meta_secondary.add_modifier(Modifier::BOLD),
            marker: p.meta_meter.add_modifier(Modifier::BOLD),
            selected: p.search_match.add_modifier(Modifier::BOLD),
            error: p.error,
        }
    }
}

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
    s: &PickerStyle,
    title: &str,
    filter: &Input,
    list_width: Option<u16>,
) -> PickerAreas {
    Clear.render(area, frame.buffer_mut());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(s.border)
        .title(Span::styled(format!(" {title} "), s.title));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [filter_area, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(inner);

    let filter_line = Line::from(vec![
        Span::styled(" / ", s.accent),
        Span::styled(" ", Style::default()),
        Span::styled(filter.value().to_string(), s.text),
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
    s: &PickerStyle,
    rows: Vec<ListItem>,
    selected_row: Option<usize>,
    empty_message: &str,
) {
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(format!(" {empty_message}"), s.muted)),
            area,
        );
        return;
    }

    let list = List::new(rows)
        .highlight_style(s.selected)
        .highlight_symbol("> ");
    let mut state = ListState::default().with_selected(selected_row);
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn draw_footer(frame: &mut Frame, area: Rect, line: Line) {
    frame.render_widget(Paragraph::new(line), area);
}

pub fn hints<'a>(s: &PickerStyle, hints: &[(&'a str, &'a str)]) -> Line<'a> {
    let mut spans = vec![Span::raw(" ")];
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", s.muted));
        }
        spans.push(Span::styled(*key, s.key));
        spans.push(Span::styled(format!(" {action}"), s.muted));
    }
    Line::from(spans)
}
