use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Widget};
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;
use crate::ui::utils::centered_rect;

pub fn draw_display_type_select(frame: &mut Frame, app: &mut App) {
    if let AppMode::DisplayTypeSelect { items, state } = &mut app.mode {
        let area = frame.area();

        let list_items = items
            .iter()
            .map(|dt| ListItem::new(format!("{:?}", dt)))
            .collect::<Vec<_>>();

        let percent_x = 60;
        let popup_area = centered_rect(percent_x, area, items.len() as u16 + 2);

        Clear.render(popup_area, frame.buffer_mut());

        let list_widget = List::new(list_items)
            .block(Block::default().title("Select Type").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list_widget, popup_area, state);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_all_display_types_visible() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_display_type_select();

        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal
            .draw(|frame| draw_display_type_select(frame, &mut app))
            .unwrap();
        let screen = terminal.backend().to_string();

        let AppMode::DisplayTypeSelect { items, .. } = &app.mode else {
            unreachable!()
        };
        for item in items {
            assert!(
                screen.contains(&format!("{:?}", item)),
                "{:?} not shown",
                item
            );
        }

        let mut small = Terminal::new(TestBackend::new(40, 5)).unwrap();
        small
            .draw(|frame| draw_display_type_select(frame, &mut app))
            .unwrap();
    }
}
