use humantime::format_duration;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, ScrollbarState, Wrap};
use ratatui::Frame;
use tui_input::Input;

use crate::command::{Command, CommandOutput};
use crate::config::AppConfig;
use crate::mode::{ContentCache, DiffMode};
use crate::ui::diffs;
use crate::ui::picker::{self, PickerStyle};
use crate::ui::utils::{highlight_lines, scrollbar};

const MATCH_CONTEXT_ROWS: u16 = 2;

pub struct ContentView<'a> {
    pub selected_idx: usize,
    pub diff_mode: DiffMode,
    pub search: &'a Input,
    pub search_focused: bool,
    pub focused: bool,
    pub scroll_offset: &'a mut u16,
    pub max_scroll: &'a mut u16,
    pub scrollbar_state: &'a mut ScrollbarState,
    pub current_match: &'a mut usize,
    pub match_count: &'a mut usize,
    pub jump_to_match: &'a mut bool,
    pub cache: &'a mut Option<ContentCache>,
}

fn wrapped(paragraph: Paragraph, wrap: bool) -> Paragraph {
    if wrap {
        paragraph.wrap(Wrap { trim: false })
    } else {
        paragraph
    }
}

fn title<'a>(
    command: &Command,
    current: Option<&CommandOutput>,
    previous: Option<&CommandOutput>,
    diff_mode: DiffMode,
    s: &PickerStyle,
) -> (Line<'a>, Line<'a>) {
    let mode = match diff_mode {
        DiffMode::None => "Plain".to_string(),
        other => format!("{other} diff"),
    };
    let time = |o: &CommandOutput| o.time.format("%H:%M:%S").to_string();
    let compared = match (current, previous) {
        (Some(current), Some(previous)) if diff_mode != DiffMode::None => {
            format!(" {} vs {} · {mode} ", time(current), time(previous))
        }
        (Some(current), _) => format!(" {} · {mode} ", time(current)),
        (None, _) => format!(" no output yet · {mode} "),
    };

    (
        Line::from(vec![
            Span::styled(format!(" {} ", command.exec), s.title),
            Span::styled(
                format!("· every {} ", format_duration(command.interval)),
                s.muted,
            ),
        ]),
        Line::from(Span::styled(compared, s.muted)).right_aligned(),
    )
}

fn key_hints<'a>(view: &ContentView, s: &PickerStyle) -> Line<'a> {
    let searching = !view.search.value().is_empty();
    let back = if searching {
        ("esc", "clear")
    } else {
        ("esc", "back")
    };

    let hints: Vec<(&str, &str)> = if view.search_focused {
        vec![("type", "search"), ("enter", "keep"), ("esc", "cancel")]
    } else if view.focused {
        let mut hints = vec![("↑↓", "scroll"), ("→", "history"), ("tab", "diff")];
        if searching {
            hints.push(("n/N", "match"));
        }
        hints.extend([("/", "search"), ("w", "wrap"), back]);
        hints
    } else {
        vec![
            ("↑↓", "select run"),
            ("←", "output"),
            ("tab", "diff"),
            ("/", "search"),
            back,
        ]
    };

    let mut line = picker::hints(s, &hints);
    line.push_span(Span::raw(" "));
    line
}

fn draw_search_strip(
    frame: &mut Frame,
    area: Rect,
    search: &Input,
    focused: bool,
    current_match: usize,
    match_count: usize,
    s: &PickerStyle,
) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("/ ", s.accent),
            Span::styled(search.value().to_string(), s.text),
        ])),
        area,
    );

    let status = match (search.value().is_empty(), match_count) {
        (true, _) => Span::raw(""),
        (false, 0) => Span::styled("no matches", s.error),
        (false, count) => Span::styled(format!("match {} of {count}", current_match + 1), s.muted),
    };
    frame.render_widget(Paragraph::new(Line::from(status).right_aligned()), area);

    if focused {
        let cursor_x = area.x + 2 + search.visual_cursor() as u16;
        if cursor_x < area.right() {
            frame.set_cursor_position((cursor_x, area.y));
        }
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    config: &AppConfig,
    command: &Command,
    view: ContentView,
) {
    let p = &config.theme.palette;
    let s = PickerStyle::from_palette(p);
    let current_len = command.output_history.len();

    let data_idx = current_len
        .saturating_sub(1)
        .saturating_sub(view.selected_idx);
    let prev_data_idx = data_idx.checked_sub(1);

    let current_output = command.output_history.get(data_idx);
    let previous_output = prev_data_idx.and_then(|idx| command.output_history.get(idx));

    let key = (
        current_output.map(|c| c.time),
        previous_output.map(|c| c.time),
        view.diff_mode,
    );
    if view.cache.as_ref().is_none_or(|cache| cache.key != key) {
        let current_text = current_output.map_or("", |c| &c.output);
        let previous_text = previous_output.map_or("", |c| &c.output);
        *view.cache = Some(ContentCache {
            key,
            lines: diffs::owned(diffs::render_diff(
                &config.theme,
                current_text,
                previous_text,
                view.diff_mode,
            )),
        });
    }
    let cached = view.cache.as_ref().map_or(&[][..], |cache| &cache.lines);

    let query = view.search.value();
    let current_style = p
        .search_match
        .add_modifier(Modifier::REVERSED | Modifier::BOLD);
    let (lines, match_rows) = highlight_lines(
        diffs::borrowed(cached),
        query,
        p.search_match,
        Some((*view.current_match, current_style)),
    );
    *view.match_count = match_rows.len();
    if *view.current_match >= match_rows.len() {
        *view.current_match = 0;
    }

    let border_style = if view.focused {
        p.border_active
    } else {
        p.border_inactive
    };
    let (left_title, right_title) =
        title(command, current_output, previous_output, view.diff_mode, &s);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::left(2))
        .title(left_title)
        .title(right_title)
        .title_bottom(key_hints(&view, &s));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let show_search = view.search_focused || !query.is_empty();
    let (text_area, search_area) = if show_search {
        let [text, search] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner);
        (text, Some(search))
    } else {
        (inner, None)
    };

    if *view.jump_to_match {
        if let Some(&row) = match_rows.get(*view.current_match) {
            let visual_row = if row == 0 {
                0
            } else if config.wrap {
                wrapped(Paragraph::new(lines[..row].to_vec()), true).line_count(text_area.width)
            } else {
                row
            };
            *view.scroll_offset = (visual_row as u16).saturating_sub(MATCH_CONTEXT_ROWS);
        }
        *view.jump_to_match = false;
    }

    let paragraph = wrapped(Paragraph::new(lines), config.wrap);
    let content_length = paragraph.line_count(text_area.width) as u16;
    let max_scroll = content_length.saturating_sub(text_area.height);
    *view.scroll_offset = (*view.scroll_offset).min(max_scroll);

    frame.render_widget(paragraph.scroll((*view.scroll_offset, 0)), text_area);
    scrollbar::widget(
        frame,
        text_area,
        p,
        content_length,
        view.max_scroll,
        view.scrollbar_state,
        view.scroll_offset,
    );

    if let Some(search_area) = search_area {
        draw_search_strip(
            frame,
            search_area,
            view.search,
            view.search_focused,
            *view.current_match,
            *view.match_count,
            &s,
        );
    }
}
