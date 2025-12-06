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
use std::collections::HashMap;
use std::io;

use super::utils::centered_rect;

pub fn manage_cursor<B: Backend>(app: &App, terminal: &mut Terminal<B>) -> io::Result<()> {
    match &app.mode {
        AppMode::CmdEdit { input } | AppMode::SessionSave { input } => {
            let term_size = terminal.size()?;
            let term_rect = Rect::new(0, 0, term_size.width, term_size.height);
            let popup_area = centered_rect(60, term_rect);

            let cursor_x = popup_area.x + input.cursor() as u16 + 1;
            let cursor_y = popup_area.y + 1;

            terminal.set_cursor_position((cursor_x, cursor_y))?;
            terminal.show_cursor()
        }
        _ => terminal.hide_cursor(),
    }
}