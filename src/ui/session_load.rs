use std::time::SystemTime;

use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;
use crate::session::SessionEntry;
use crate::ui::picker;

const META_WIDTH: usize = 18;

fn format_age(modified: Option<SystemTime>, now: SystemTime) -> String {
    let Some(age) = modified.and_then(|m| now.duration_since(m).ok()) else {
        return String::new();
    };
    let secs = age.as_secs();
    match secs {
        0..60 => "just now".to_string(),
        60..3_600 => format!("{}m ago", secs / 60),
        3_600..86_400 => format!("{}h ago", secs / 3_600),
        86_400..2_592_000 => format!("{}d ago", secs / 86_400),
        2_592_000..31_536_000 => format!("{}mo ago", secs / 2_592_000),
        _ => format!("{}y ago", secs / 31_536_000),
    }
}

fn format_size(bytes: u64) -> String {
    match bytes {
        0..1_024 => format!("{bytes} B"),
        1_024..1_048_576 => format!("{} KB", bytes / 1_024),
        _ => format!("{:.1} MB", bytes as f64 / 1_048_576.0),
    }
}

fn session_row<'a>(
    entry: &'a SessionEntry,
    width: usize,
    now: SystemTime,
    p: &crate::config::theme::Palette,
) -> ListItem<'a> {
    let name_width = width.saturating_sub(META_WIDTH + 3);
    let name: String = entry.name.chars().take(name_width).collect();
    let meta = format!(
        "{:>9} {:>8}",
        format_age(entry.modified, now),
        format_size(entry.size)
    );
    ListItem::new(Line::from(vec![
        Span::styled(format!("{name:<name_width$}"), p.meta_value),
        Span::styled(meta, p.meta_label),
    ]))
}

pub fn draw_session_list(frame: &mut Frame, app: &mut App) {
    let AppMode::SessionLoad { picker: state } = &app.mode else {
        return;
    };
    let p = &app.config.theme.palette;

    let empty_message = if state.items().is_empty() {
        "No sessions yet. Press S in normal mode to save one."
    } else {
        "No matching sessions"
    };

    let area = picker::popup_area(frame.area(), 70, state.items().len() as u16);
    let areas = picker::draw_frame(frame, area, p, "Load session", &state.filter, None);

    let now = SystemTime::now();
    let width = areas.list.width as usize;
    let rows: Vec<ListItem> = state
        .visible()
        .map(|(_, entry)| session_row(entry, width, now, p))
        .collect();

    picker::draw_list(
        frame,
        areas.list,
        p,
        rows,
        state.state.selected(),
        empty_message,
    );
    picker::draw_footer(
        frame,
        areas.footer,
        picker::hints(
            p,
            &[
                ("↑↓", "move"),
                ("enter", "load"),
                ("type", "filter"),
                ("esc", "cancel"),
            ],
        ),
    );
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;
    use crate::config::AppConfig;
    use crate::mode::Picker;

    fn render(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| draw_session_list(frame, app))
            .unwrap();
        terminal.backend().to_string()
    }

    fn app_with_sessions(count: usize, selected: usize) -> App {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let now = SystemTime::now();
        let mut picker = Picker::new(
            (0..count)
                .map(|i| SessionEntry {
                    file_name: format!("session-{i:02}.toml"),
                    name: format!("session-{i:02}"),
                    modified: Some(now - Duration::from_secs(3_600 * i as u64)),
                    size: 1_024 * i as u64,
                })
                .collect(),
            |s: &SessionEntry| s.name.clone(),
        );
        picker.state.select(Some(selected));
        app.mode = AppMode::SessionLoad { picker };
        app
    }

    #[test]
    fn test_all_sessions_visible_when_they_fit() {
        let mut app = app_with_sessions(10, 0);
        let screen = render(&mut app, 100, 30);
        for i in 0..10 {
            assert!(screen.contains(&format!("session-{i:02}")));
        }
    }

    #[test]
    fn test_long_session_list_scrolls_to_selection() {
        let mut app = app_with_sessions(50, 49);
        let screen = render(&mut app, 100, 20);
        assert!(screen.contains("session-49"));
    }

    #[test]
    fn test_rows_show_age_and_size() {
        let mut app = app_with_sessions(3, 0);
        let screen = render(&mut app, 100, 20);
        let row = screen.lines().find(|l| l.contains("session-02")).unwrap();
        assert!(row.contains("2h ago"), "{row}");
        assert!(row.contains("2 KB"), "{row}");
    }

    #[test]
    fn test_age_and_size_formatting() {
        let now = SystemTime::now();
        let ago = |secs| format_age(Some(now - Duration::from_secs(secs)), now);
        assert_eq!(ago(5), "just now");
        assert_eq!(ago(120), "2m ago");
        assert_eq!(ago(2 * 86_400), "2d ago");
        assert_eq!(ago(130 * 86_400), "4mo ago");
        assert_eq!(format_age(None, now), "");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(964_915), "942 KB");
        assert_eq!(format_size(3 * 1_048_576), "3.0 MB");
    }

    #[test]
    fn test_empty_session_list_explains_itself() {
        let mut app = app_with_sessions(0, 0);
        let screen = render(&mut app, 100, 20);
        assert!(screen.contains("No sessions yet"));
        render(&mut app, 10, 2);
    }
}
