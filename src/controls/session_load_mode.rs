use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;

use crate::app::App;
use crate::controls::actions::Action;
use crate::logging::{debug, error, info};
use crate::mode::AppMode;
use crate::session::load_session_by_name;

pub async fn handle_session_load_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let AppMode::SessionLoad { items, state } = &mut app.mode {
        if let Event::Key(key_event) = event {
            if key_event.kind == event::KeyEventKind::Press {
                let key_combination: KeyCombination = KeyCombination::from(key_event);
                if let Some(action) = app.config.keybindings.get(&key_combination) {
                    match action {
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
            }
        }
    }
    Ok(())
}
