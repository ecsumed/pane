use std::io;

use crokey::KeyCombination;
use crossterm::event::{self, Event};
use tui_input::backend::crossterm::EventHandler;

use crate::app::App;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::logging::debug;
use crate::mode::{AppMode, DiffMode, ObserveFocus};

pub async fn handle_observe_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::Observe {
        active_id,
        selected_history_idx,
        diff_mode,
        search_input,
        focus,
        scroll_offset,
        max_scroll,
        ..
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

    if *focus == ObserveFocus::Search {
        match action {
            Some(Action::Escape | Action::Confirm) => *focus = ObserveFocus::Content,
            _ => {
                search_input.handle_event(&event);
            }
        }
        return Ok(());
    }

    if let Some(act) = action {
        match act {
            Action::Escape | Action::Quit => {
                app.mode = AppMode::Normal;
            }

            Action::MoveLeft => *focus = ObserveFocus::Content,
            Action::MoveRight => *focus = ObserveFocus::History,
            Action::Search => *focus = ObserveFocus::Search,

            Action::MoveUp => match focus {
                ObserveFocus::History => {
                    if *selected_history_idx > 0 {
                        *selected_history_idx -= 1;
                        *scroll_offset = 0; // Reset scroll when changing history
                    }
                }
                ObserveFocus::Content => {
                    *scroll_offset = scroll_offset.saturating_sub(1);
                }
                ObserveFocus::Search => {}
            },

            Action::MoveDown => match focus {
                ObserveFocus::History => {
                    let history_len = app
                        .tasks
                        .get(active_id)
                        .map_or(0, |cmd| cmd.output_history.len());

                    if *selected_history_idx < history_len.saturating_sub(1) {
                        *selected_history_idx += 1;
                        *scroll_offset = 0;
                    }
                }
                ObserveFocus::Content => {
                    *scroll_offset = (*scroll_offset).saturating_add(1).min(*max_scroll);
                }
                ObserveFocus::Search => {}
            },

            Action::WrapToggle => {
                if *focus == ObserveFocus::Content {
                    app.config.wrap = !app.config.wrap;
                }
            }

            Action::Cycle => {
                *diff_mode = match diff_mode {
                    DiffMode::None => DiffMode::Line,
                    DiffMode::Line => DiffMode::Word,
                    DiffMode::Word => DiffMode::Char,
                    DiffMode::Char => DiffMode::None,
                };
                debug!("Cycling diff to {}", diff_mode);
            }

            Action::ScrollTop => {
                if *focus == ObserveFocus::Content {
                    *scroll_offset = 0;
                }
            }

            Action::ScrollBottom => {
                if *focus == ObserveFocus::Content {
                    *scroll_offset = *max_scroll;
                }
            }

            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::config::AppConfig;

    async fn press(app: &mut App, code: KeyCode) {
        let event = Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
        handle_observe_mode_keys(app, event).await.unwrap();
    }

    fn search_state(app: &App) -> Option<(String, &ObserveFocus)> {
        match &app.mode {
            AppMode::Observe {
                search_input,
                focus,
                ..
            } => Some((search_input.value().to_string(), focus)),
            _ => None,
        }
    }

    #[tokio::test]
    async fn test_search_accepts_bound_keys_as_text() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_observing(&app);

        press(&mut app, KeyCode::Char('/')).await;
        for c in "quiet/wg".chars() {
            press(&mut app, KeyCode::Char(c)).await;
        }

        let (value, focus) = search_state(&app).expect("left observe mode");
        assert_eq!(value, "quiet/wg");
        assert_eq!(*focus, ObserveFocus::Search);
    }

    #[tokio::test]
    async fn test_escape_leaves_search_but_stays_in_observe() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_observing(&app);

        press(&mut app, KeyCode::Char('/')).await;
        press(&mut app, KeyCode::Char('x')).await;
        press(&mut app, KeyCode::Esc).await;

        let (value, focus) = search_state(&app).expect("left observe mode");
        assert_eq!(value, "x");
        assert_eq!(*focus, ObserveFocus::Content);

        press(&mut app, KeyCode::Esc).await;
        assert!(matches!(app.mode, AppMode::Normal));
    }

    #[tokio::test]
    async fn test_history_down_on_pane_without_command() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_observing(&app);

        press(&mut app, KeyCode::Down).await;

        assert!(matches!(app.mode, AppMode::Observe { .. }));
    }

    #[tokio::test]
    async fn test_unfocused_keys_do_not_edit_search() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_observing(&app);

        press(&mut app, KeyCode::Char('x')).await;

        let (value, _) = search_state(&app).expect("left observe mode");
        assert_eq!(value, "");
    }
}
