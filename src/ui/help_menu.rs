use std::path::Path;

use humantime::format_duration;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::AppConfig;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::mode::AppMode;
use crate::ui::picker::{self, PickerStyle, Prompt};
use crate::ui::utils::scrollbar;

const SETTING_WIDTH: usize = 34;
const SETTING_LABEL_WIDTH: usize = 19;
const KEY_WIDTH: usize = 9;
const COLUMN_GAP: usize = 3;

const MODES: [(KeyMode, &str); 8] = [
    (KeyMode::Global, "Global"),
    (KeyMode::Normal, "Normal"),
    (KeyMode::CmdEdit, "Command edit"),
    (KeyMode::Observe, "Observe"),
    (KeyMode::DisplayTypeSelect, "Display select"),
    (KeyMode::SessionLoad, "Session load"),
    (KeyMode::SessionSave, "Session save"),
    (KeyMode::Help, "Help"),
];

type Cell = Vec<Span<'static>>;

fn on_off(value: bool) -> String {
    if value { "on" } else { "off" }.to_string()
}

pub fn home_relative(path: &Path) -> String {
    std::env::var_os("HOME")
        .and_then(|home| path.strip_prefix(home).ok())
        .map_or(path.display().to_string(), |rest| {
            format!("~/{}", rest.display())
        })
}

fn humanize(action: &Action) -> String {
    let name = format!("{action:?}");
    let mut words = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            words.push(' ');
        }
        words.extend(c.to_lowercase());
    }
    words
}

fn short_settings(c: &AppConfig) -> Vec<(&'static str, String)> {
    let t = &c.theme;
    vec![
        ("Interval", format_duration(c.interval).to_string()),
        ("Timeout", format_duration(c.timeout).to_string()),
        ("Max history", c.max_history.to_string()),
        ("Default display", format!("{:?}", c.default_display)),
        ("Wrap", on_off(c.wrap)),
        ("Zen", on_off(c.zen)),
        ("Beep", on_off(c.beep)),
        ("Exit on error", on_off(c.err_exit)),
        ("Exit on change", on_off(c.chg_exit)),
        (
            "Log level",
            c.log_level.clone().unwrap_or("off".to_string()),
        ),
        ("Theme", format!("{:?}", t.mode).to_lowercase()),
        ("Collapse borders", on_off(t.collapse_borders)),
        ("Show state", on_off(t.show_state)),
        ("Show last updated", on_off(t.show_last_updated)),
        ("Show display type", on_off(t.show_display_type)),
        ("Show history meter", on_off(t.show_history_meter)),
        ("Show status bar", on_off(t.show_status_bar)),
        ("Inline deletions", on_off(t.show_inline_deletions)),
    ]
}

fn long_settings(c: &AppConfig) -> Vec<(&'static str, String)> {
    vec![
        ("Sessions", home_relative(&c.sessions_dir)),
        ("Logs", home_relative(&c.logs_dir)),
    ]
}

fn setting_cell(label: &str, value: String, s: &PickerStyle) -> Cell {
    vec![
        Span::styled(format!("{label:<SETTING_LABEL_WIDTH$}"), s.muted),
        Span::styled(value, s.text),
    ]
}

fn cell_width(cell: &Cell) -> usize {
    cell.iter().map(|span| span.content.chars().count()).sum()
}

fn join_row(cells: Vec<Cell>, width: usize) -> Line<'static> {
    let mut spans = Vec::new();
    let last = cells.len().saturating_sub(1);
    for (i, cell) in cells.into_iter().enumerate() {
        let used = cell_width(&cell);
        spans.extend(cell);
        if i < last {
            spans.push(Span::raw(" ".repeat(width.saturating_sub(used))));
        }
    }
    Line::from(spans)
}

fn grid(cells: Vec<Cell>, cell_width: usize, width: usize) -> Vec<Line<'static>> {
    let columns = (width / cell_width).max(1);
    let mut cells = cells.into_iter().peekable();
    let mut lines = Vec::new();
    while cells.peek().is_some() {
        let row: Vec<Cell> = cells.by_ref().take(columns).collect();
        lines.push(join_row(row, cell_width));
    }
    lines
}

fn key_groups(c: &AppConfig, query: &str, s: &PickerStyle) -> Vec<Vec<Cell>> {
    MODES
        .iter()
        .filter_map(|(mode, name)| {
            let bindings = c.keybindings.get(mode)?;
            let group_matches = name.to_lowercase().contains(query);

            let mut entries: Vec<(String, String)> = bindings
                .iter()
                .map(|(key, action)| (key.to_string(), humanize(action)))
                .filter(|(key, action)| {
                    group_matches || key.to_lowercase().contains(query) || action.contains(query)
                })
                .collect();
            if entries.is_empty() {
                return None;
            }
            entries.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

            let mut group = vec![vec![Span::styled(
                name.to_uppercase(),
                s.accent.add_modifier(Modifier::BOLD),
            )]];
            group.extend(entries.into_iter().map(|(key, action)| {
                vec![
                    Span::styled(format!("{key:<KEY_WIDTH$}"), s.key),
                    Span::styled(action, s.text),
                ]
            }));
            Some(group)
        })
        .collect()
}

fn key_columns(groups: Vec<Vec<Cell>>, width: usize) -> Vec<Line<'static>> {
    let column_width = groups.iter().flatten().map(cell_width).max().unwrap_or(0) + COLUMN_GAP;
    let count = (width / column_width.max(1)).max(1);
    let mut columns: Vec<Vec<Cell>> = vec![Vec::new(); count];

    for group in groups {
        let column = columns
            .iter_mut()
            .min_by_key(|column| column.len())
            .expect("at least one column");
        if !column.is_empty() {
            column.push(Vec::new());
        }
        column.extend(group);
    }

    let rows = columns.iter().map(Vec::len).max().unwrap_or(0);
    (0..rows)
        .map(|row| {
            let cells = columns
                .iter()
                .map(|column| column.get(row).cloned().unwrap_or_default())
                .collect();
            join_row(cells, column_width)
        })
        .collect()
}

fn heading(text: &str, s: &PickerStyle) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        s.title.add_modifier(Modifier::UNDERLINED),
    ))
}

pub fn help_lines(c: &AppConfig, query: &str, width: usize) -> Vec<Line<'static>> {
    let s = PickerStyle::from_palette(&c.theme.palette);
    let query = query.to_lowercase();
    let matches = |label: &str, value: &str| {
        label.to_lowercase().contains(&query) || value.to_lowercase().contains(&query)
    };

    let short: Vec<Cell> = short_settings(c)
        .into_iter()
        .filter(|(label, value)| matches(label, value))
        .map(|(label, value)| setting_cell(label, value, &s))
        .collect();
    let long: Vec<Line> = long_settings(c)
        .into_iter()
        .filter(|(label, value)| matches(label, value))
        .map(|(label, value)| Line::from(setting_cell(label, value, &s)))
        .collect();
    let groups = key_groups(c, &query, &s);

    let mut lines = Vec::new();
    if !short.is_empty() || !long.is_empty() {
        lines.push(heading("Settings", &s));
        lines.extend(grid(short, SETTING_WIDTH, width));
        lines.extend(long);
    }
    if !groups.is_empty() {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        lines.push(heading("Keys", &s));
        lines.extend(key_columns(groups, width));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No settings or keys match",
            s.muted,
        )));
    }
    lines
}

pub fn draw_help_menu(frame: &mut Frame, c: &AppConfig, mode: &mut AppMode) {
    let AppMode::Help {
        scroll_offset,
        max_scroll,
        scrollbar_state,
        filter,
        filtering,
    } = mode
    else {
        return;
    };
    let s = PickerStyle::from_palette(&c.theme.palette);

    let area = picker::popup_area(frame.area(), 120, frame.area().height);
    let prompt = Prompt {
        symbol: "/",
        input: filter,
        focused: *filtering,
        placeholder: "press / to filter settings and keys",
    };
    let areas = picker::draw_frame(frame, area, &s, "Help", prompt, None);

    let body = areas.list;
    let text_width = body.width.saturating_sub(2);
    let lines = help_lines(c, filter.value(), text_width as usize);
    let content_length = lines.len() as u16;
    *scroll_offset = (*scroll_offset).min(content_length.saturating_sub(body.height));

    frame.render_widget(
        Paragraph::new(lines).scroll((*scroll_offset, 0)),
        body.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 0,
        }),
    );
    scrollbar::widget(
        frame,
        body,
        &c.theme.palette,
        content_length,
        max_scroll,
        scrollbar_state,
        scroll_offset,
    );

    let hints: Vec<(&str, &str)> = if *filtering {
        vec![("type", "filter"), ("enter", "keep"), ("esc", "clear")]
    } else {
        let close = if filter.value().is_empty() {
            ("esc", "close")
        } else {
            ("esc", "clear filter")
        };
        vec![
            ("↑↓", "scroll"),
            ("g/G", "top/bottom"),
            ("/", "filter"),
            close,
        ]
    };
    picker::draw_footer(frame, areas.footer, picker::hints(&s, &hints));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lines: &[Line]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_settings_grid_and_key_columns_fit_the_width() {
        let config = AppConfig::default();
        for width in [40, 80, 118] {
            let lines = help_lines(&config, "", width);
            for line in &lines {
                let used: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
                assert!(used <= width.max(SETTING_WIDTH * 2), "{used} > {width}");
            }
        }

        let wide = text(&help_lines(&config, "", 118));
        let interval_row = wide.lines().find(|l| l.contains("Interval")).unwrap();
        assert!(interval_row.contains("Timeout") && interval_row.contains("Max history"));
        assert!(wide.contains("KEYS") || wide.contains("Keys"));
        assert!(wide.contains("split horizontal"));
        for line in wide.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();
            assert!(
                !words
                    .iter()
                    .any(|w| w.ends_with("modeOBSERVE") || *w == "horizontalg"),
                "columns ran together: {line}"
            );
        }
        let normal_row = wide
            .lines()
            .find(|l| l.contains("enter display select mode"))
            .unwrap();
        let after = normal_row
            .split("enter display select mode")
            .nth(1)
            .unwrap();
        assert!(after.is_empty() || after.starts_with("   "), "{normal_row}");
    }

    #[test]
    fn test_filter_narrows_settings_and_keys() {
        let config = AppConfig::default();
        let filtered = text(&help_lines(&config, "split", 118));
        assert!(filtered.contains("split horizontal"));
        assert!(filtered.contains("split vertical"));
        assert!(!filtered.contains("Settings"));
        assert!(!filtered.contains("save session"));

        let by_mode = text(&help_lines(&config, "observe", 118));
        assert!(by_mode.contains("OBSERVE"));
        assert!(by_mode.contains("next match"));

        let nothing = text(&help_lines(&config, "zzzz", 118));
        assert_eq!(nothing, "No settings or keys match");
    }

    #[test]
    fn test_humanize_actions() {
        assert_eq!(humanize(&Action::SplitHorizontal), "split horizontal");
        assert_eq!(humanize(&Action::EnterCmdMode), "enter cmd mode");
        assert_eq!(humanize(&Action::Quit), "quit");
    }
}
