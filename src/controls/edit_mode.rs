use std::{io, mem};

use crokey::KeyCombination;
use crossterm::event::{self, Event};
use ratatui::widgets::ListState;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::app::{App, AppControl};
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::logging::{debug, warn};
use crate::mode::AppMode;
use crate::shell_history::ShellHistoryManager;

fn update_suggestions(
    input: &mut Input,
    history: &mut ShellHistoryManager,
    state: &mut ListState,
    suggestions: &mut Vec<String>,
) {
    let input_value = input.value().to_string();

    *suggestions = history.filter(&input_value);
    state.select(None);
    debug!("Suggestions found: {}", suggestions.len());
}

pub async fn handle_editing_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::CmdEdit {
        input,
        state,
        suggestions,
        history,
    } = &mut app.mode
    else {
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
                let exec = if let Some(index) = state.selected() {
                    suggestions[index].clone()
                } else {
                    input.value().to_string()
                };

                let id = app.pane_manager.active_pane_id;
                if let Err(e) = app.app_control_tx.send(AppControl::SetCommand(id, exec)) {
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
                let index = state.selected().unwrap_or(0);
                if let Some(suggestion) = suggestions.get(index).cloned() {
                    let current_input = std::mem::take(input);

                    let updated_input = current_input.with_value(suggestion);

                    let _ = mem::replace(input, updated_input);
                    update_suggestions(input, history, state, suggestions);
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

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::config::AppConfig;

    fn app_with_history(commands: &[&str]) -> App {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::CmdEdit {
            input: Input::default(),
            state: ListState::default(),
            suggestions: Vec::new(),
            history: ShellHistoryManager::from_commands(
                commands.iter().map(|c| c.to_string()).collect(),
            ),
        };
        app
    }

    async fn press(app: &mut App, code: KeyCode) {
        let event = Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
        handle_editing_mode_keys(app, event).await.unwrap();
    }

    async fn type_text(app: &mut App, text: &str) {
        for c in text.chars() {
            press(app, KeyCode::Char(c)).await;
        }
    }

    fn submitted(app: &mut App) -> Option<String> {
        match app.app_control_rx.try_recv() {
            Ok(AppControl::SetCommand(_, exec)) => Some(exec),
            _ => None,
        }
    }

    #[tokio::test]
    async fn test_enter_runs_typed_command_over_history_match() {
        let mut app = app_with_history(&["kubectl get pods -A"]);

        type_text(&mut app, "kubectl get pods").await;
        press(&mut app, KeyCode::Enter).await;

        assert_eq!(submitted(&mut app).as_deref(), Some("kubectl get pods"));
    }

    #[tokio::test]
    async fn test_enter_runs_suggestion_after_selecting_it() {
        let mut app = app_with_history(&["kubectl get pods -A"]);

        type_text(&mut app, "kubectl").await;
        press(&mut app, KeyCode::Down).await;
        press(&mut app, KeyCode::Enter).await;

        assert_eq!(submitted(&mut app).as_deref(), Some("kubectl get pods -A"));
    }

    #[tokio::test]
    async fn test_tab_completes_then_typing_continues() {
        let mut app = app_with_history(&["kubectl get pods"]);

        type_text(&mut app, "kub").await;
        press(&mut app, KeyCode::Tab).await;
        type_text(&mut app, " -n logging").await;
        press(&mut app, KeyCode::Enter).await;

        assert_eq!(
            submitted(&mut app).as_deref(),
            Some("kubectl get pods -n logging")
        );
    }
}
