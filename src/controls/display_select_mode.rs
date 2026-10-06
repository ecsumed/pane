use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;

use crate::app::{App, AppControl};
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::logging::{info, warn};
use crate::mode::AppMode;

pub async fn handle_display_type_select_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::DisplayTypeSelect { picker } = &mut app.mode else {
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
            if let Some(&display_type) = picker.selected() {
                let id = app.pane_manager.active_pane_id;

                info!("Changing to display: {:?}", display_type);
                if let Err(e) = app
                    .app_control_tx
                    .send(AppControl::SetDisplay(id, display_type))
                {
                    warn!("Failed to send AppControl::SetDisplay: {}", e);
                }
                app.mode = AppMode::Normal;
            }
        }
        Some(Action::Escape) => app.mode = AppMode::Normal,
        _ => picker.handle_filter_event(&event),
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
        handle_display_type_select_keys(app, event).await.unwrap();
    }

    #[tokio::test]
    async fn test_typing_filters_and_esc_cancels() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_display_type_select(None);

        press(&mut app, KeyCode::Char('q')).await;
        let AppMode::DisplayTypeSelect { picker } = &app.mode else {
            panic!("q closed the picker");
        };
        assert_eq!(picker.filter.value(), "q");

        press(&mut app, KeyCode::Esc).await;
        assert!(matches!(app.mode, AppMode::Normal));
    }

    #[tokio::test]
    async fn test_enter_applies_filtered_selection() {
        let mut app = App::new(AppConfig::default(), Vec::new());
        app.mode = AppMode::new_display_type_select(None);

        for c in "counter".chars() {
            press(&mut app, KeyCode::Char(c)).await;
        }
        press(&mut app, KeyCode::Enter).await;

        assert!(matches!(app.mode, AppMode::Normal));
        assert!(matches!(
            app.app_control_rx.try_recv(),
            Ok(AppControl::SetDisplay(_, crate::ui::DisplayType::Counter))
        ));
    }
}
