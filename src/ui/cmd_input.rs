use ratatui::prelude::{Frame, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget};

use crate::mode::AppMode;
use crate::App;

pub fn draw_input_popup(frame: &mut Frame, app: &mut App) {
    if let AppMode::CmdEdit {
        input,
        state,
        suggestions,
        ..
    } = &mut app.mode
    {
        let frame_area = frame.area();
        let percent_x = 60;

        let input_area_width = frame_area.width * percent_x / 100;
        let input_area_x = (frame_area.width.saturating_sub(input_area_width)) / 2;
        let input_area_y = (frame_area.height.saturating_sub(3)) / 2;

        let input_area =
            Rect::new(input_area_x, input_area_y, input_area_width, 3).intersection(frame_area);

        Clear.render(input_area, frame.buffer_mut());

        let num_suggestions = suggestions.len() as u16;
        let suggestions_height = num_suggestions + if num_suggestions > 0 { 1 } else { 0 };

        let suggestions_area = Rect::new(
            input_area.x,
            input_area.bottom(),
            input_area.width,
            suggestions_height,
        )
        .intersection(frame_area);

        Clear.render(suggestions_area, frame.buffer_mut());

        let input_widget = Paragraph::new(input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Command"),
        );
        frame.render_widget(input_widget, input_area);

        let cursor_x = input_area.x + 1 + input.cursor() as u16;
        let cursor_y = input_area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));

        if !suggestions.is_empty() && suggestions_area.height > 0 {
            let list_items: Vec<ListItem> = suggestions
                .iter()
                .map(|s| ListItem::new(s.as_str()))
                .collect();

            let suggestions_list = List::new(list_items)
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            frame.render_stateful_widget(suggestions_list, suggestions_area, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::ListState;
    use ratatui::Terminal;
    use tui_input::Input;

    use super::*;
    use crate::config::AppConfig;
    use crate::shell_history::ShellHistoryManager;

    #[test]
    fn test_suggestions_fit_short_terminal() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::CmdEdit {
            input: Input::default().with_value("kubectl".to_string()),
            state: ListState::default(),
            suggestions: (0..10).map(|i| format!("kubectl get pods {i}")).collect(),
            history: ShellHistoryManager::from_commands(Vec::new()),
        };

        for height in [1, 3, 10, 15] {
            let mut terminal = Terminal::new(TestBackend::new(80, height)).unwrap();
            terminal
                .draw(|frame| draw_input_popup(frame, &mut app))
                .unwrap();
        }
    }
}
