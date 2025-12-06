use crate::command::Command;
use crate::mode::{AppMode};
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::App;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Backend, Frame, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Widget};
use ratatui::Terminal;
use std::collections::HashMap;
use std::io;

use super::utils::centered_rect;

pub fn draw_session_save_popup(frame: &mut Frame, app: &App) {
    let popup_area = centered_rect(60, frame.area());

    Clear.render(popup_area, frame.buffer_mut());

    if let AppMode::SessionSave { input } = &app.mode {
        let input_widget = Paragraph::new(input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Session Name"),
        );
        
        frame.render_widget(input_widget, popup_area);
    }
}