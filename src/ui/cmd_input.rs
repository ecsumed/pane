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

        let input_area = Rect::new(input_area_x, input_area_y, input_area_width, 3);

        Clear.render(input_area, frame.buffer_mut());

        let num_suggestions = suggestions.len() as u16;
        let suggestions_height = num_suggestions + if num_suggestions > 0 { 1 } else { 0 };

        let suggestions_area = Rect::new(
            input_area.x,
            input_area.y + input_area.height,
            input_area.width,
            suggestions_height,
        );

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
