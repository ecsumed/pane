use crate::mode::AppMode;
use crate::App;
use ratatui::layout::Position;
use ratatui::prelude::{Backend, Frame};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use super::utils::centered_rect;

pub fn draw_session_save_popup(frame: &mut Frame, app: &App) {
    if let AppMode::SessionSave { input } = &app.mode {
        let popup_area = centered_rect(60, frame.area(), 3);

        Clear.render(popup_area, frame.buffer_mut());

        frame.set_cursor_position(Position::new(
            popup_area.x + 1 + input.cursor() as u16,
            popup_area.y + 1,
        ));

        let input_widget = Paragraph::new(input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Session Name"),
        );

        frame.render_widget(input_widget, popup_area);
    }
}
