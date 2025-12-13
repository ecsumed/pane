use crokey::KeyCombination;
use crokey::crossterm::event::{self, Event};

use crate::app::{App, AppControl};
use crate::controls::actions::Action;
use crate::mode::AppMode;
use std::io;

use crate::logging::{info, warn};

pub async fn handle_display_type_select_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let AppMode::DisplayTypeSelect { items, state } = &mut app.mode {
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
                                let display_type = items[selected].clone();
                                let id = app.pane_manager.active_pane_id;

                                info!("Changing to display: {:?}", display_type);
                                if let Err(e) = app
                                    .app_control_tx
                                    .send(AppControl::SetDisplay(id, display_type))
                                    .await
                                {
                                    warn!("Failed to send AppControl::SetDisplay: {}", e);
                                }
                            }
                            app.mode = AppMode::Normal;
                        }
                        Action::Escape => {
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
