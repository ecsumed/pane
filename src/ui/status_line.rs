use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::mode::AppMode;
use crate::ui::picker::PickerStyle;

pub fn mode_output(mode: &AppMode) -> String {
    format!(" {}", mode)
}

pub fn draw_status_line(frame: &mut Frame, area: Rect, app: &App) {
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(25)]).areas(area);

    let mut left_content = Line::from(mode_output(&app.mode));
    if let Some(notice) = app.current_notice() {
        let s = PickerStyle::from_palette(&app.config.theme.palette);
        let style = if notice.is_error { s.error } else { s.accent };
        left_content.push_span(Span::raw("  "));
        left_content.push_span(Span::styled(notice.text.clone(), style));
    }
    let left_widget = Paragraph::new(left_content);

    let right_content = Line::from("?-Help q/esc - Quit/Back ");
    let right_widget = Paragraph::new(right_content).alignment(Alignment::Right);

    frame.render_widget(left_widget, left_area);
    frame.render_widget(right_widget, right_area);
}
