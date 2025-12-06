use crate::app::{App, AppControl};
use crate::mode::AppMode;
use crossterm::event::{self, Event, KeyCode};
use std::io;
use tui_input::backend::crossterm::EventHandler;
use crate::logging::{debug, info, warn, error};
use crate::session::save_session_by_name;


pub async fn handle_session_save_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let AppMode::SessionSave { input } = &mut app.mode {
        if let Event::Key(key_event) = event {
            if key_event.kind == event::KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Enter => {
                        let session_name = input.value().to_string();
                        info!("Saving session to {}", session_name);
                        if let Err(e) = save_session_by_name(app, &session_name) {
                            error!("Saving session: {}", e);
                        } else {
                            info!("Session saved successfully!");
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