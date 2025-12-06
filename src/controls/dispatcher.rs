use crate::app::App;
use crate::mode::AppMode;
use crossterm::event::Event;
use std::io;

use super::normal_mode::handle_normal_mode_keys;
use super::edit_mode::handle_editing_mode_keys;
use super::session_mode::handle_session_load_keys;

pub async fn handle_event(app: &mut App, event: Event) -> io::Result<()> {
    match &mut app.mode {
        AppMode::Normal => {
            handle_normal_mode_keys(app, event).await?
        } 
        
        AppMode::CmdEdit { .. } => {
            handle_editing_mode_keys(app, event).await?
        }

        AppMode::SessionLoad { .. } => {
            handle_session_load_keys(app, event).await?
        }

        // TODO: Temporary
        _ => handle_normal_mode_keys(app, event).await?,
    }
    Ok(())
}