use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Rect;
use std::cmp::min;

pub fn centered_rect(percent_x: u16, frame_area: Rect, min_height: u16) -> Rect {
    let final_height = min(min_height, frame_area.height);

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(final_height),
            Constraint::Min(0),
        ])
        .split(frame_area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical_chunks[1]);

    horizontal_chunks[1]
}
