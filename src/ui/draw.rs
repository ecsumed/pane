use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::Frame;

use crate::mode::AppMode;
use crate::ui::cmd_input::draw_input_popup;
use crate::ui::display_select::draw_display_type_select;
use crate::ui::help_menu::draw_help_menu;
use crate::ui::observe::draw_observe_mode;
use crate::ui::panes;
use crate::ui::session_load::draw_session_list;
use crate::ui::session_save::draw_session_save_popup;
use crate::ui::status_line::draw_status_line;
use crate::App;

pub fn draw_ui(app: &mut App, frame: &mut Frame) {
    let [main_area, status_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    // Main modes
    match &mut app.mode {
        AppMode::Observe { .. } => draw_observe_mode(frame, main_area, &app.tasks,  &mut app.mode),
        _ => panes::draw(frame, main_area, app.config.zen,&app.pane_manager, &app.tasks),
    }

    draw_status_line(frame, status_area,  &app);

    // Popups and overlays
    match &mut app.mode {
        AppMode::CmdEdit { .. } => draw_input_popup(frame, app),
        AppMode::SessionLoad { .. } => draw_session_list(frame, app),
        AppMode::SessionSave { .. } => draw_session_save_popup(frame, app),
        AppMode::DisplayTypeSelect { .. } => draw_display_type_select(frame, app),
        AppMode::Help { .. } => draw_help_menu(frame, app),
        _ => ()
    }
}
