use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;
use tui_input::backend::crossterm::EventHandler;

use crate::app::App;
use crate::controls::actions::Action;
use crate::logging::{error, info};
use crate::mode::AppMode;
use crate::session::save_session_by_name;

pub async fn handle_session_save_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let AppMode::SessionSave { input } = &mut app.mode {
        if let Event::Key(key_event) = event {
            if key_event.kind == event::KeyEventKind::Press {
                let key_combination: KeyCombination = KeyCombination::from(key_event);
                if let Some(action) = app.config.keybindings.get(&key_combination) {
                    match action {
                        Action::Confirm => {
                            let session_name = input.value().to_string();
                            info!("Saving session to {}", session_name);
                            if let Err(e) = save_session_by_name(app, &session_name) {
                                error!("Saving session: {}", e);
                            } else {
                                info!("Session saved successfully!");
                            }
                            app.mode = AppMode::Normal;
                        }
                        Action::Escape => {
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
        } else {
            input.handle_event(&event);
        }
    }

    Ok(())
}
