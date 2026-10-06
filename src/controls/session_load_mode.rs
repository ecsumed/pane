use std::io;

use crokey::crossterm::event::{self, Event, KeyCode};
use crokey::KeyCombination;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::app::App;
use crate::config::AppConfig;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::logging::{error, info};
use crate::mode::Picker;
use crate::mode::{AppMode, SessionPrompt};
use crate::session::{
    delete_session, fetch_sessions, load_session_by_name, rename_session, SessionEntry,
};

fn refresh_sessions(picker: &mut Picker<SessionEntry>, config: &AppConfig) {
    let sessions = fetch_sessions(config).unwrap_or_default();
    picker.set_items(sessions, |s: &SessionEntry| s.name.clone());
}

pub async fn handle_session_load_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::SessionLoad {
        picker,
        prompt,
        previews,
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

    match std::mem::take(prompt) {
        SessionPrompt::ConfirmDelete => {
            if key_event.code == KeyCode::Char('y') {
                if let Some(entry) = picker.selected().cloned() {
                    match delete_session(&app.config, &entry.file_name) {
                        Ok(()) => {
                            info!("Deleted session {}", entry.file_name);
                            previews.remove(&entry.file_name);
                            refresh_sessions(picker, &app.config);
                        }
                        Err(e) => *prompt = SessionPrompt::Error(format!("Failed to delete: {e}")),
                    }
                }
            }
            app.mode.load_session_preview(&app.config);
            return Ok(());
        }
        SessionPrompt::Rename(mut input) => {
            match action {
                Some(Action::Confirm) => {
                    if let Some(entry) = picker.selected().cloned() {
                        match rename_session(&app.config, &entry.file_name, input.value()) {
                            Ok(new_file_name) => {
                                info!("Renamed session {} to {}", entry.file_name, new_file_name);
                                previews.remove(&entry.file_name);
                                refresh_sessions(picker, &app.config);
                                picker.clear_filter();
                                if let Some(index) = picker
                                    .items()
                                    .iter()
                                    .position(|s| s.file_name == new_file_name)
                                {
                                    picker.select_item(index);
                                }
                            }
                            Err(e) => {
                                *prompt = SessionPrompt::Error(format!("Couldn't rename: {e}"))
                            }
                        }
                    }
                }
                Some(Action::Escape) => {}
                _ => {
                    input.handle_event(&event);
                    *prompt = SessionPrompt::Rename(input);
                }
            }
            app.mode.load_session_preview(&app.config);
            return Ok(());
        }
        SessionPrompt::Error(_) | SessionPrompt::None => {}
    }

    match action {
        Some(Action::MoveUp) => picker.move_up(),
        Some(Action::MoveDown) => picker.move_down(),
        Some(Action::Confirm) => {
            if let Some(session_filename) = picker.selected().map(|s| s.file_name.clone()) {
                info!("Loading session: {}", session_filename);

                match load_session_by_name(app, &session_filename) {
                    Ok(()) => {
                        info!("Session loaded successfully!");
                        app.notify(
                            format!(
                                "Loaded session {}",
                                session_filename.trim_end_matches(".toml")
                            ),
                            false,
                        );
                        app.mode = AppMode::Normal;
                    }
                    Err(e) => {
                        error!("Error loading session: {}", e);
                        if let AppMode::SessionLoad { prompt, .. } = &mut app.mode {
                            *prompt = SessionPrompt::Error(format!("Failed to load: {e}"));
                        }
                    }
                }
            }
        }
        Some(Action::Delete) => {
            if picker.selected().is_some() {
                *prompt = SessionPrompt::ConfirmDelete;
            }
        }
        Some(Action::Rename) => {
            if let Some(entry) = picker.selected() {
                *prompt = SessionPrompt::Rename(Input::default().with_value(entry.name.clone()));
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
        press_with(app, code, KeyModifiers::NONE).await;
    }

    async fn press_with(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
        let event = Event::Key(KeyEvent::new(code, modifiers));
        handle_session_load_keys(app, event).await.unwrap();
    }

    fn app_with_session_files(test: &str, names: &[&str]) -> (App, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("pane-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for name in names {
            fs::write(dir.join(format!("{name}.toml")), "").unwrap();
        }
        let config = AppConfig {
            sessions_dir: dir.clone(),
            ..AppConfig::default()
        };
        let mut app = App::new(config, Vec::new());
        app.mode = AppMode::new_session_load(&app);
        (app, dir)
    }

    fn names(app: &App) -> Vec<String> {
        let AppMode::SessionLoad { picker, .. } = &app.mode else {
            panic!("picker closed");
        };
        let mut names: Vec<String> = picker.items().iter().map(|s| s.name.clone()).collect();
        names.sort();
        names
    }

    fn selected(app: &App) -> Option<String> {
        let AppMode::SessionLoad { picker, .. } = &app.mode else {
            panic!("picker closed");
        };
        picker.selected().map(|s| s.name.clone())
    }

    #[tokio::test]
    async fn test_delete_needs_confirmation() {
        let (mut app, dir) = app_with_session_files("delete", &["alpha", "beta"]);
        let target = selected(&app).unwrap();

        press_with(&mut app, KeyCode::Char('d'), KeyModifiers::CONTROL).await;
        press(&mut app, KeyCode::Char('n')).await;
        assert_eq!(names(&app), vec!["alpha", "beta"]);

        press_with(&mut app, KeyCode::Char('d'), KeyModifiers::CONTROL).await;
        press(&mut app, KeyCode::Char('y')).await;
        let remaining = names(&app);
        assert_eq!(remaining.len(), 1);
        assert!(!remaining.contains(&target));
        assert!(!dir.join(format!("{target}.toml")).exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn test_rename_session() {
        let (mut app, dir) = app_with_session_files("rename", &["alpha", "beta"]);
        app.mode = AppMode::new_session_load(&app);
        for c in "beta".chars() {
            press(&mut app, KeyCode::Char(c)).await;
        }

        press_with(&mut app, KeyCode::Char('r'), KeyModifiers::CONTROL).await;
        for _ in 0.."beta".len() {
            press(&mut app, KeyCode::Backspace).await;
        }
        for c in "istio".chars() {
            press(&mut app, KeyCode::Char(c)).await;
        }
        press(&mut app, KeyCode::Enter).await;

        assert_eq!(names(&app), vec!["alpha", "istio"]);
        assert_eq!(selected(&app).as_deref(), Some("istio"));
        assert!(dir.join("istio.toml").exists());

        press_with(&mut app, KeyCode::Char('r'), KeyModifiers::CONTROL).await;
        for _ in 0.."istio".len() {
            press(&mut app, KeyCode::Backspace).await;
        }
        for c in "alpha".chars() {
            press(&mut app, KeyCode::Char(c)).await;
        }
        press(&mut app, KeyCode::Enter).await;
        let AppMode::SessionLoad { prompt, .. } = &app.mode else {
            panic!("picker closed");
        };
        assert!(matches!(prompt, SessionPrompt::Error(m) if m.contains("already exists")));
        assert!(dir.join("istio.toml").exists());

        fs::remove_dir_all(&dir).unwrap();
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
        assert!(matches!(prompt, SessionPrompt::None));

        fs::remove_dir_all(&dir).unwrap();
    }
}
