use ratatui::widgets::ListItem;
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;
use crate::ui::picker;

pub fn draw_display_type_select(frame: &mut Frame, app: &mut App) {
    let AppMode::DisplayTypeSelect { picker: state } = &app.mode else {
        return;
    };
    let p = &app.config.theme.palette;

    let rows: Vec<ListItem> = state
        .visible()
        .map(|(_, dt)| ListItem::new(format!("{:?}", dt)))
        .collect();

    let area = picker::popup_area(frame.area(), 40, state.items().len() as u16);
    let areas = picker::draw_frame(frame, area, p, "Display", &state.filter);
    picker::draw_list(
        frame,
        areas.list,
        p,
        rows,
        state.state.selected(),
        "No matching display types",
    );
    picker::draw_footer(
        frame,
        areas.footer,
        picker::hints(
            p,
            &[
                ("↑↓", "move"),
                ("enter", "apply"),
                ("type", "filter"),
                ("esc", "cancel"),
            ],
        ),
    );
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use strum::IntoEnumIterator;

    use super::*;
    use crate::config::AppConfig;
    use crate::ui::DisplayType;

    fn render(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| draw_display_type_select(frame, app))
            .unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn test_all_display_types_visible() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_display_type_select();

        let screen = render(&mut app, 100, 30);
        for item in DisplayType::iter() {
            assert!(
                screen.contains(&format!("{:?}", item)),
                "{:?} not shown",
                item
            );
        }

        render(&mut app, 40, 5);
        render(&mut app, 10, 2);
    }

    #[test]
    fn test_filter_narrows_display_types() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_display_type_select();
        if let AppMode::DisplayTypeSelect { picker } = &mut app.mode {
            for c in "chart".chars() {
                let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
                picker.handle_filter_event(&Event::Key(key));
            }
        }

        let screen = render(&mut app, 100, 30);
        assert!(screen.contains("LineChart"));
        assert!(!screen.contains("RawText"));
    }
}
