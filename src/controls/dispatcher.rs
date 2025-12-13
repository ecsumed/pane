use std::io;

use crossterm::event::Event;

use super::display_select_mode::handle_display_type_select_keys;
use super::edit_mode::handle_editing_mode_keys;
use super::normal_mode::handle_normal_mode_keys;
use super::session_load_mode::handle_session_load_keys;
use super::session_save_mode::handle_session_save_keys;
use crate::app::App;
use crate::controls::help_mode::handle_help_keys;
use crate::mode::AppMode;

pub async fn handle_event(app: &mut App, event: Event) -> io::Result<()> {
    match &mut app.mode {
        AppMode::Normal => handle_normal_mode_keys(app, event).await?,

        AppMode::CmdEdit { .. } => handle_editing_mode_keys(app, event).await?,

        AppMode::SessionLoad { .. } => handle_session_load_keys(app, event).await?,

        AppMode::SessionSave { .. } => handle_session_save_keys(app, event).await?,

        AppMode::DisplayTypeSelect { .. } => handle_display_type_select_keys(app, event).await?,

        AppMode::Help { .. } => handle_help_keys(app, event).await?,
    }
    Ok(())
}
