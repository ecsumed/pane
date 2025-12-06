use crate::command::Command;
use crate::pane::{PaneKey, PaneManager, PaneNodeData};
use crate::mode::AppMode;
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

use crate::ui::pane::draw_panes;
use crate::ui::cmd_input::draw_input_popup;
use crate::ui::session_load::draw_session_list;
use crate::ui::session_save::draw_session_save_popup;
use crate::ui::utils::centered_rect;

pub fn draw_ui(app: &mut App, frame: &mut Frame) {
    draw_panes(frame, &app.pane_manager, &app.tasks);

    // Popups and overlays
    match &mut app.mode {
        AppMode::CmdEdit { .. } => {
            draw_input_popup(frame, app)
        }
        AppMode::SessionLoad { .. } => {
            draw_session_list(frame, app)
        }
        AppMode::SessionSave { .. } => {
            draw_session_save_popup(frame, app)
        }
        _ => ()
    }
}
