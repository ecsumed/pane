use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;

use crate::app::App;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::logging::{error, info};
use crate::mode::{AppMode, SessionPrompt};
use crate::session::load_session_by_name;

pub async fn handle_session_load_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::SessionLoad { picker, prompt, .. } = &mut app.mode else {
        return Ok(());
    };

    let Event::Key(key_event) = event else {
        return Ok(());
    };
    if key_event.kind != event::KeyEventKind::Press {
        return Ok(());
    }

    let key_comb: KeyCombination = KeyCombination::from(key_event);
    *prompt = SessionPrompt::None;

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

    match action {
        Some(Action::MoveUp) => picker.move_up(),
        Some(Action::MoveDown) => picker.move_down(),
        Some(Action::Confirm) => {
            if let Some(session_filename) = picker.selected().map(|s| s.file_name.clone()) {
                info!("Loading session: {}", session_filename);

                match load_session_by_name(app, &session_filename) {
                    Ok(()) => {
                        info!("Session loaded successfully!");
                        app.mode = AppMode::Normal;
                    }
                    Err(e) => {
                        error!("Error loading session: {}", e);
                        if let AppMode::SessionLoad { prompt, .. } = &mut app.mode {
                            *prompt = SessionPrompt::Error(e.to_string());
                        }
                    }
                }
            }
        }
        Some(Action::Escape) => app.mode = AppMode::Normal,
        _ => picker.handle_filter_event(&event),
    }

    app.mode.load_session_preview(&app.config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::config::AppConfig;

    async fn press(app: &mut App, code: KeyCode) {
        let event = Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
        handle_session_load_keys(app, event).await.unwrap();
    }

    #[tokio::test]
    async fn test_failed_load_keeps_picker_open_with_error() {
        let dir = std::env::temp_dir().join(format!("pane-load-error-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("broken.toml"), "this is not a session").unwrap();

        let config = AppConfig {
            sessions_dir: dir.clone(),
            ..AppConfig::default()
        };
        let mut app = App::new(config, Vec::new());
        app.mode = AppMode::new_session_load(&app);

        press(&mut app, KeyCode::Enter).await;
        let AppMode::SessionLoad { prompt, .. } = &app.mode else {
            panic!("picker closed after failed load");
        };
        assert!(matches!(prompt, SessionPrompt::Error(_)));

        press(&mut app, KeyCode::Down).await;
        let AppMode::SessionLoad { prompt, .. } = &app.mode else {
            panic!("picker closed");
        };
        assert_eq!(*prompt, SessionPrompt::None);

        fs::remove_dir_all(&dir).unwrap();
    }
}
