use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;
use ratatui::layout::Direction;

use super::actions::Action;
use crate::app::{App, AppControl};
use crate::command::CommandControl;
use crate::logging::{error, info, warn};
use crate::mode::AppMode;
use crate::pane::CardinalDirection;
use crate::session::{load_latest_session, save_session};

pub async fn handle_normal_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let Event::Key(key_event) = event {
        if key_event.kind == event::KeyEventKind::Press {
            let key_combination: KeyCombination = KeyCombination::from(key_event);
            if let Some(action) = app.config.keybindings.get(&key_combination) {
                match action {
                    Action::Quit => {
                        app.exit();
                    }
                    Action::SplitHorizontal => {
                        app.pane_manager.split_pane(Direction::Horizontal);
                    }
                    Action::SplitVertical => {
                        app.pane_manager.split_pane(Direction::Vertical);
                    }
                    Action::EnterCmdMode => {
                        app.mode = AppMode::new_cmd_edit();
                    }
                    Action::CyclePanes => app.pane_manager.cycle_panes(),
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
                    Action::KillPane => {
                        let id = app.pane_manager.active_pane_id;
                        if let Err(e) = app
                            .app_control_tx
                            .send(AppControl::SendControl(id, CommandControl::Stop))
                            .await
                        {
                            warn!("Failed to send AppControl::SendControl: {}", e);
                        }
                        app.pane_manager.kill_pane();
                    }
                    Action::Confirm => {
                        let id = app.pane_manager.active_pane_id;
                        if let Err(e) = app
                            .app_control_tx
                            .send(AppControl::SendControl(id, CommandControl::Execute))
                            .await
                        {
                            warn!("Failed to send AppControl::SendControl: {}", e);
                        }
                    }
                    Action::Pause => {
                        let id = app.pane_manager.active_pane_id;
                        if let Err(e) = app
                            .app_control_tx
                            .send(AppControl::SendControl(id, CommandControl::Pause))
                            .await
                        {
                            warn!("Failed to send AppControl::SendControl: {}", e);
                        }
                    }
                    Action::Resume => {
                        let id = app.pane_manager.active_pane_id;
                        if let Err(e) = app
                            .app_control_tx
                            .send(AppControl::SendControl(id, CommandControl::Resume))
                            .await
                        {
                            warn!("Failed to send AppControl::SendControl: {}", e);
                        }
                    }
                    Action::IntervalIncrease => {
                        let id = app.pane_manager.active_pane_id;
                        if let Err(e) = app
                            .app_control_tx
                            .send(AppControl::SendControl(
                                id,
                                CommandControl::IntervalIncrease,
                            ))
                            .await
                        {
                            warn!("Failed to send AppControl::SendControl: {}", e);
                        }
                    }
                    Action::IntervalDecrease => {
                        let id = app.pane_manager.active_pane_id;
                        if let Err(e) = app
                            .app_control_tx
                            .send(AppControl::SendControl(
                                id,
                                CommandControl::IntervalDecrease,
                            ))
                            .await
                        {
                            warn!("Failed to send AppControl::SendControl: {}", e);
                        }
                    }
                    Action::SaveSession => {
                        info!("Saving session...");
                        if let Err(e) = save_session(&app) {
                            error!("Error saving session: {}", e);
                        } else {
                            info!("Session saved successfully!");
                        }
                    }
                    Action::LoadLatestSession => {
                        info!("Loading latest session...");
                        if let Err(e) = load_latest_session(app) {
                            error!("Error loading session: {}", e);
                        } else {
                            info!("Session loaded successfully!");
                        }
                    }
                    Action::EnterSessionLoadMode => {
                        info!("Fetching sessions mode");
                        app.mode = AppMode::new_session_load(app);
                    }
                    Action::EnterSessionSaveMode => {
                        info!("Saving sessions mode");
                        app.mode = AppMode::new_session_save();
                    }
                    Action::EnterDisplaySelectMode => {
                        info!("Display select mode");
                        app.mode = AppMode::new_display_type_select();
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
                    _ => (),
                }
                //}
            }
        }
    }
    Ok(())
}
