use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;

pub fn mode_output(mode: &AppMode) -> String {
    match mode {
        AppMode::Observe { diff_mode, .. } => {
            format!(" {} -> {} <tab> - cycle", mode, diff_mode)
        }
        _ => format!(" {}", mode),
    }
}

pub fn draw_status_line(frame: &mut Frame, area: Rect, app: &App) {
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(25)]).areas(area);

    let left_content = Line::from(mode_output(&app.mode));
    let left_widget = Paragraph::new(left_content);

    let right_content = Line::from("?-Help q/esc - Quit/Back ");
    let right_widget = Paragraph::new(right_content).alignment(Alignment::Right);

    frame.render_widget(left_widget, left_area);
    frame.render_widget(right_widget, right_area);
}
