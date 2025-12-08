use crate::command::Command;
use crate::mode::AppMode;
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::App;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Backend, Frame, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};
use ratatui::Terminal;
use std::cmp::min;
use std::collections::HashMap;
use std::io;

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