use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;
use tui_input::backend::crossterm::EventHandler;

use crate::app::App;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::logging::{error, info};
use crate::mode::AppMode;
use crate::session::{save_session, save_session_by_name};

pub async fn handle_session_save_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::SessionSave { input, .. } = &mut app.mode else {
        return Ok(());
    };

    let Event::Key(key_event) = event else {
        return Ok(());
    };
    if key_event.kind != event::KeyEventKind::Press {
        return Ok(());
    }

    let key_comb: KeyCombination = KeyCombination::from(key_event);

    let action = app
        .config
        .keybindings
        .get(&current_context)
        .and_then(|map| map.get(&key_comb))
        .or_else(|| {
            app.config
                .keybindings
                .get(&KeyMode::Global)
                .and_then(|map| map.get(&key_comb))
        });

    if let Some(act) = action {
        match act {
            Action::Confirm => {
                let session_name = input.value().trim().to_string();
                info!("Saving session to {}", session_name);
                let saved = if session_name.is_empty() {
                    save_session(app)
                } else {
                    save_session_by_name(app, &session_name)
                };
                match saved {
                    Ok(name) => app.notify(format!("Saved session {name}"), false),
                    Err(e) => {
                        error!("Saving session: {}", e);
                        app.notify(format!("Couldn't save session: {e}"), true);
                    }
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
    Ok(())
}
