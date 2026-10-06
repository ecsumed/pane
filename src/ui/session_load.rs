use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Widget};
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;
use crate::ui::utils::centered_rect;

pub fn draw_session_list(frame: &mut Frame, app: &mut App) {
    if let AppMode::SessionLoad { items, state } = &mut app.mode {
        let area = frame.area();

        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut item = ListItem::new(s.as_str());

                if state.selected() == Some(i) {
                    item = item.style(Style::default().add_modifier(Modifier::BOLD));
                }
                item
            })
            .collect();

        let percent_x = 60;
        let popup_area = centered_rect(percent_x, area, (items.len() as u16).max(1) + 2);

        Clear.render(popup_area, frame.buffer_mut());

        let list_widget = List::new(list_items)
            .block(Block::default().title("Load Session").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list_widget, popup_area, state);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::ListState;
    use ratatui::Terminal;

    use super::*;
    use crate::config::AppConfig;

    fn render(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| draw_session_list(frame, app))
            .unwrap();
        terminal.backend().to_string()
    }

    fn app_with_sessions(count: usize, selected: usize) -> App {
        let mut app = App::new(AppConfig::default(), Vec::new());
        let mut state = ListState::default();
        state.select(Some(selected));
        app.mode = AppMode::SessionLoad {
            items: (0..count).map(|i| format!("session-{i:02}.toml")).collect(),
            state,
        };
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

        let mut empty = app_with_sessions(0, 0);
        render(&mut empty, 40, 3);
    }
}
