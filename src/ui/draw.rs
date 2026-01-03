use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::Frame;
use ratatui::widgets::{Clear, Widget};

use crate::mode::AppMode;
use crate::ui::cmd_input::draw_input_popup;
use crate::ui::display_select::draw_display_type_select;
use crate::ui::help_menu::draw_help_menu;
use crate::ui::observe;
use crate::ui::panes;
use crate::ui::session_load::draw_session_list;
use crate::ui::session_save::draw_session_save_popup;
use crate::ui::status_line::draw_status_line;
use crate::App;

pub fn draw_ui(app: &mut App, frame: &mut Frame) {
    let mut constraints = vec![Constraint::Min(0)];

    if app.config.theme.show_status_bar {
        constraints.push(Constraint::Length(1));
    }

    let areas = Layout::vertical(constraints).split(frame.area());
    let main_area = areas[0];

    if app.config.theme.show_status_bar {
        let status_area = areas[1];
        draw_status_line(frame, status_area, &app);
    }

    Clear.render(main_area, frame.buffer_mut());

    // Main modes
    match &mut app.mode {
        AppMode::Observe { .. } => {
            observe::draw(frame, main_area, &app.config, &app.tasks, &mut app.mode)
        }
        _ => panes::draw(frame, main_area, &app.config, &app.pane_manager, &app.tasks),
    }

    // Popups and overlays
    match &mut app.mode {
        AppMode::CmdEdit { .. } => draw_input_popup(frame, app),
        AppMode::SessionLoad { .. } => draw_session_list(frame, app),
        AppMode::SessionSave { .. } => draw_session_save_popup(frame, app),
        AppMode::DisplayTypeSelect { .. } => draw_display_type_select(frame, app),
        AppMode::Help { .. } => draw_help_menu(frame, app),
        _ => (),
    }
}
