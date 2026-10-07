use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;
use ratatui::layout::Direction;

use super::actions::Action;
use crate::app::{App, AppControl};
use crate::command::CommandControl;
use crate::controls::KeyMode;
use crate::logging::{error, info, warn};
use crate::mode::AppMode;
use crate::pane::CardinalDirection;
use crate::session::{load_latest_session, save_session};

pub async fn handle_normal_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

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
            Action::Quit | Action::Escape => {
                app.exit();
            }
            Action::SplitHorizontal => {
                app.pane_manager.split_pane(Direction::Horizontal);
            }
            Action::SplitVertical => {
                app.pane_manager.split_pane(Direction::Vertical);
            }
            Action::EnterCmdMode => {
                let id = app.pane_manager.active_pane_id;
                app.mode = AppMode::new_cmd_edit(app.tasks.get(&id).map(|c| c.exec.clone()));
            }
            Action::Cycle => app.pane_manager.cycle_panes(),
            Action::MoveUp => {
                app.pane_manager
                    .change_active(&CardinalDirection::Up, app.pane_area);
            }
            Action::MoveDown => {
                app.pane_manager
                    .change_active(&CardinalDirection::Down, app.pane_area);
            }
            Action::MoveLeft => {
                app.pane_manager
                    .change_active(&CardinalDirection::Left, app.pane_area);
            }
            Action::MoveRight => {
                app.pane_manager
                    .change_active(&CardinalDirection::Right, app.pane_area);
            }
            Action::KillPane => app.kill_active_pane(),
            Action::Confirm => {
                let id = app.pane_manager.active_pane_id;
                if let Err(e) = app
                    .app_control_tx
                    .send(AppControl::SendControl(id, CommandControl::Execute))
                {
                    warn!("Failed to send AppControl::SendControl: {}", e);
                }
            }
            Action::Pause => {
                let id = app.pane_manager.active_pane_id;
                if let Err(e) = app
                    .app_control_tx
                    .send(AppControl::SendControl(id, CommandControl::Pause))
                {
                    warn!("Failed to send AppControl::SendControl: {}", e);
                }
            }
            Action::Resume => {
                let id = app.pane_manager.active_pane_id;
                if let Err(e) = app
                    .app_control_tx
                    .send(AppControl::SendControl(id, CommandControl::Resume))
                {
                    warn!("Failed to send AppControl::SendControl: {}", e);
                }
            }
            Action::IntervalIncrease => {
                let id = app.pane_manager.active_pane_id;
                if let Err(e) = app.app_control_tx.send(AppControl::SendControl(
                    id,
                    CommandControl::IntervalIncrease,
                )) {
                    warn!("Failed to send AppControl::SendControl: {}", e);
                }
            }
            Action::IntervalDecrease => {
                let id = app.pane_manager.active_pane_id;
                if let Err(e) = app.app_control_tx.send(AppControl::SendControl(
                    id,
                    CommandControl::IntervalDecrease,
                )) {
                    warn!("Failed to send AppControl::SendControl: {}", e);
                }
            }
            Action::SaveSession => {
                info!("Saving session...");
                match save_session(app) {
                    Ok(name) => app.notify(format!("Saved session {name}"), false),
                    Err(e) => {
                        error!("Error saving session: {}", e);
                        app.notify(format!("Couldn't save session: {e}"), true);
                    }
                }
            }
            Action::LoadLatestSession => {
                info!("Loading latest session...");
                match load_latest_session(app) {
                    Ok(name) => app.notify(format!("Loaded session {name}"), false),
                    Err(e) => {
                        error!("Error loading session: {}", e);
                        app.notify(format!("Couldn't load session: {e}"), true);
                    }
                }
            }
            Action::EnterSessionLoadMode => {
                info!("Fetching sessions mode");
                app.mode = AppMode::new_session_load(app);
            }
            Action::EnterSessionSaveMode => {
                info!("Saving sessions mode");
                app.mode = AppMode::new_session_save(app);
            }
            Action::EnterDisplaySelectMode => {
                info!("Display select mode");
                let id = app.pane_manager.active_pane_id;
                app.mode =
                    AppMode::new_display_type_select(app.tasks.get(&id).map(|c| c.display_type));
            }
            Action::PaneIncreaseVertical => {
                app.pane_manager.resize_pane(&CardinalDirection::Right, 1);
            }
            Action::PaneDecreaseVertical => {
                app.pane_manager.resize_pane(&CardinalDirection::Left, -1);
            }
            Action::PaneIncreaseHorizontal => {
                app.pane_manager.resize_pane(&CardinalDirection::Down, 1);
            }
            Action::PaneDecreaseHorizontal => {
                app.pane_manager.resize_pane(&CardinalDirection::Up, -1);
            }
            Action::EnterHelpMode => {
                info!("Help mode");
                app.mode = AppMode::new_help();
            }
            Action::EnterObserveMode => {
                info!("Observe mode");
                app.mode = AppMode::new_observing(app);
            }
            Action::WrapToggle => {
                app.config.wrap = !app.config.wrap;
            }
            Action::ZenToggle => {
                app.config.zen = !app.config.zen;
            }
            Action::Execute => {
                let id = app.pane_manager.active_pane_id;
                if let Err(e) = app
                    .app_control_tx
                    .send(AppControl::SendControl(id, CommandControl::Execute))
                {
                    warn!("Failed to send AppControl::Execute: {}", e);
                }
            }
            _ => (),
        }
    }
    Ok(())
}
