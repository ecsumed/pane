use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;

use crate::app::App;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::logging::{error, info};
use crate::mode::AppMode;
use crate::session::load_session_by_name;

pub async fn handle_session_load_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::SessionLoad { picker } = &mut app.mode else {
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

    match action {
        Some(Action::MoveUp) => picker.move_up(),
        Some(Action::MoveDown) => picker.move_down(),
        Some(Action::Confirm) => {
            if let Some(session_filename) = picker.selected().map(|s| s.file_name.clone()) {
                info!("Loading session: {}", session_filename);

                if let Err(e) = load_session_by_name(app, &session_filename) {
                    error!("Error loading session: {}", e);
                } else {
                    info!("Session loaded successfully!");
                }
                app.mode = AppMode::Normal;
            }
        }
        Some(Action::Escape) => app.mode = AppMode::Normal,
        _ => picker.handle_filter_event(&event),
    }
    Ok(())
}
