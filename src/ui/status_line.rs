use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::Frame;
use ratatui::widgets::{Block, Paragraph};

use crate::app::App;


pub fn draw_status_line(frame: &mut Frame, area: Rect, app: &App) {
    let status_content = Line::from(format!(" {}", app.mode));

    let widget = Paragraph::new(status_content)
        .block(Block::default());

    frame.render_widget(widget, area);
}
