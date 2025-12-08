use crate::mode::AppMode;
use crate::App;
use ratatui::prelude::Frame;

use crate::ui::cmd_input::draw_input_popup;
use crate::ui::display_select::draw_display_type_select;
use crate::ui::pane::draw_panes;
use crate::ui::session_load::draw_session_list;
use crate::ui::session_save::draw_session_save_popup;

pub fn draw_ui(app: &mut App, frame: &mut Frame) {
    draw_panes(frame, &app.pane_manager, &app.tasks);

    // Popups and overlays
    match &mut app.mode {
        AppMode::CmdEdit { .. } => draw_input_popup(frame, app),
        AppMode::SessionLoad { .. } => draw_session_list(frame, app),
        AppMode::SessionSave { .. } => draw_session_save_popup(frame, app),
        AppMode::DisplayTypeSelect { .. } => draw_display_type_select(frame, app),
        _ => (),
    }
}
