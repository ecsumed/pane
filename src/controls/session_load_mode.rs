use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;

use crate::app::App;
use crate::controls::KeyMode;
use crate::controls::actions::Action;
use crate::logging::{debug, error, info};
use crate::mode::AppMode;
use crate::session::load_session_by_name;

pub async fn handle_session_load_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::SessionLoad {
        items, 
        state,
        ..
    } = &mut app.mode else {
        return Ok(());
    };

    let Event::Key(key_event) = event else {
        return Ok(());
    };
    if key_event.kind != event::KeyEventKind::Press {
        return Ok(());
    }

    let key_comb: KeyCombination = KeyCombination::from(key_event);

    let action = app.config.keybindings
        .get(&current_context)
        .and_then(|map| map.get(&key_comb))
        .or_else(|| {
            app.config.keybindings
                .get(&KeyMode::Global)
                .and_then(|map| map.get(&key_comb))
        });


    if let Some(act) = action {
        match act { 
            Action::MoveUp => {
                if let Some(selected) = state.selected() {
                    let next = if selected == 0 {
                        items.len() - 1
                    } else {
                        selected - 1
                    };
                    state.select(Some(next));
                }
            }
            Action::MoveDown => {
                if let Some(selected) = state.selected() {
                    let next = (selected + 1) % items.len();
                    state.select(Some(next));
                }
            }
            Action::Confirm => {
                if let Some(selected) = state.selected() {
                    let session_filename = items[selected].clone();

                    info!("Loading session: {}", session_filename);

                    if let Err(e) = load_session_by_name(app, &session_filename) {
                        error!("Error loading session: {}", e);
                    } else {
                        info!("Session loaded successfully!");
                    }
                }
                app.mode = AppMode::Normal;
            }
            Action::Escape | Action::Quit => {
                app.mode = AppMode::Normal;
            }
            _ => {}
        }
    }
    Ok(())
}
