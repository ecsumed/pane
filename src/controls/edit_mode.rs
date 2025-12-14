use std::{io, mem};

use crokey::KeyCombination;
use crossterm::event::{self, Event};
use ratatui::widgets::ListState;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::app::{App, AppControl};
use crate::controls::KeyMode;
use crate::controls::actions::Action;
use crate::history::HistoryManager;
use crate::logging::{debug, warn};
use crate::mode::AppMode;

fn update_suggestions(
    input: &mut Input,
    history: &mut HistoryManager,
    state: &mut ListState,
    suggestions: &mut Vec<String>,
) {
    let input_value = input.value().to_string();

    *suggestions = history.filter(&input_value);

    if !suggestions.is_empty() {
        state.select(Some(0));
        debug!("Suggestions are empty...");
    } else {
        state.select(None);
        debug!("Suggestions found: {}", suggestions.len());
    }
}

pub async fn handle_editing_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::CmdEdit {
        input,
        state,
        suggestions,
        history 
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
            Action::Confirm => {
                let exec = if let Some(index) = state.selected() {
                    suggestions[index].clone()
                } else {
                    input.value().to_string()
                };

                let id = app.pane_manager.active_pane_id;
                if let Err(e) = app
                    .app_control_tx
                    .send(AppControl::SetCommand(id, exec))
                    .await
                {
                    warn!("Failed to send AppControl::SetCommand: {}", e);
                }
                app.mode = AppMode::Normal;
            }
            Action::Escape => {
                app.mode = AppMode::Normal;
            }
            Action::MoveUp => {
                if !suggestions.is_empty() {
                    let i = match state.selected() {
                        Some(i) => {
                            if i == 0 {
                                suggestions.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    state.select(Some(i));
                }
            }
            Action::MoveDown => {
                if !suggestions.is_empty() {
                    let i = match state.selected() {
                        Some(i) => {
                            if i >= suggestions.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    state.select(Some(i));
                }
            }
            Action::TabComplete => {
                if let Some(index) = state.selected() {
                    if let Some(suggestion) = suggestions.get(index).cloned() {
                        let current_input =
                            mem::replace(input, tui_input::Input::default());

                        let updated_input = current_input.with_value(suggestion);

                        let _ = mem::replace(input, updated_input);
                    }
                }
            }
            _ => {
                input.handle_event(&event);
                update_suggestions(input, history, state, suggestions);
            }
        }
    } else {
        input.handle_event(&event);
        update_suggestions(input, history, state, suggestions);
    }
    Ok(())
}
