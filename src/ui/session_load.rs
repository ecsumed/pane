use ratatui::widgets::ListItem;
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;
use crate::ui::picker;

pub fn draw_session_list(frame: &mut Frame, app: &mut App) {
    let AppMode::SessionLoad { picker: state } = &app.mode else {
        return;
    };
    let p = &app.config.theme.palette;

    let rows: Vec<ListItem> = state
        .visible()
        .map(|(_, name)| ListItem::new(name.as_str()))
        .collect();

    let empty_message = if state.items().is_empty() {
        "No sessions yet. Press S in normal mode to save one."
    } else {
        "No matching sessions"
    };

    let area = picker::popup_area(frame.area(), 70, state.items().len() as u16);
    let areas = picker::draw_frame(frame, area, p, "Load session", &state.filter);
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
        let mut picker = Picker::new(
            (0..count).map(|i| format!("session-{i:02}.toml")).collect(),
            |s: &String| s.clone(),
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
            assert!(screen.contains(&format!("session-{i:02}.toml")));
        }
    }

    #[test]
    fn test_long_session_list_scrolls_to_selection() {
        let mut app = app_with_sessions(50, 49);
        let screen = render(&mut app, 100, 20);
        assert!(screen.contains("session-49.toml"));
    }

    #[test]
    fn test_empty_session_list_explains_itself() {
        let mut app = app_with_sessions(0, 0);
        let screen = render(&mut app, 100, 20);
        assert!(screen.contains("No sessions yet"));
        render(&mut app, 10, 2);
    }
}
