use crate::app::{App, AppControl};
use crate::mode::AppMode;
use crossterm::event::{self, Event, KeyCode};
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tracing::warn;

pub async fn handle_editing_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let AppMode::CmdEdit { input } = &mut app.mode {
        if let Event::Key(key_event) = event {
            if key_event.kind == event::KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Enter => {
                        let id = app.pane_manager.active_pane_id;
                        let exec = input.value().to_string();
                        if let Err(e) = app
                            .app_control_tx
                            .send(AppControl::SetCommand(id, exec))
                            .await
                        {
                            warn!("Failed to send AppControl::SetCommand: {}", e);
                        }
                        app.mode = AppMode::Normal;
                    }
                    KeyCode::Esc => {
                        app.mode = AppMode::Normal;
                    }
                    _ => {
                        input.handle_event(&event);
                    }
                }
            } else {
                input.handle_event(&event);
            }
        } else {
            input.handle_event(&event);
        }
    }

    Ok(())
}