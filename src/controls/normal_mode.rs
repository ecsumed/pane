use crate::app::{App, AppControl};
use crate::command::CommandControl;
use crate::mode::AppMode;
use crate::pane::CardinalDirection;
use crate::session::{load_latest_session, save_session};
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::Direction;
use std::io;
use tracing::{error, info, warn};


pub async fn handle_normal_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let Event::Key(key_event) = event {
        if key_event.kind == event::KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => {
                    app.exit();
                }
                KeyCode::Char('h') => {
                    app.pane_manager.split_pane(Direction::Horizontal);
                }
                KeyCode::Char('v') => {
                    app.pane_manager.split_pane(Direction::Vertical);
                }
                KeyCode::Char('c') => {
                    app.mode = AppMode::new_cmd_edit();
                }

                // NAVIGATION
                KeyCode::Tab => {
                    app.pane_manager.cycle_panes();
                }
                KeyCode::Up => {
                    app.pane_manager.change_active(&CardinalDirection::Up, app.pane_area);
                }
                KeyCode::Down => {
                    app.pane_manager.change_active(&CardinalDirection::Down, app.pane_area);
                }
                KeyCode::Left => {
                    app.pane_manager.change_active(&CardinalDirection::Left, app.pane_area);
                }
                KeyCode::Right => {
                    app.pane_manager.change_active(&CardinalDirection::Right, app.pane_area);
                }

                KeyCode::Char('x') => {
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
                KeyCode::Char(' ') => {
                    let id = app.pane_manager.active_pane_id;
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SendControl(id, CommandControl::Execute))
                        .await
                    {
                        warn!("Failed to send AppControl::SendControl: {}", e);
                    }
                }
                KeyCode::Char('p') => {
                    let id = app.pane_manager.active_pane_id;
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SendControl(id, CommandControl::Pause))
                        .await
                    {
                        warn!("Failed to send AppControl::SendControl: {}", e);
                    }
                }
                KeyCode::Char('r') => {
                    let id = app.pane_manager.active_pane_id;
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SendControl(id, CommandControl::Resume))
                        .await
                    {
                        warn!("Failed to send AppControl::SendControl: {}", e);
                    }
                }
                KeyCode::Char('i') => {
                    let id = app.pane_manager.active_pane_id;
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SendControl(
                            id,
                            CommandControl::IncreaseInterval,
                        ))
                        .await
                    {
                        warn!("Failed to send AppControl::SendControl: {}", e);
                    }
                }
                KeyCode::Char('d') => {
                    let id = app.pane_manager.active_pane_id;
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SendControl(
                            id,
                            CommandControl::DecreaseInterval,
                        ))
                        .await
                    {
                        warn!("Failed to send AppControl::SendControl: {}", e);
                    }
                }
                KeyCode::Char('s') => {
                    info!("Saving session...");
                    if let Err(e) = save_session(&app) {
                        error!("Error saving session: {}", e);
                    } else {
                        info!("Session saved successfully!");
                    }
                }
                KeyCode::Char('l') => {
                    info!("Loading latest session...");
                    if let Err(e) = load_latest_session(app) {
                        error!("Error loading session: {}", e);
                    } else {
                        info!("Session loaded successfully!");
                    }
                }
                KeyCode::Char('L') => {
                    info!("Fetching sessions mode");
                    app.mode = AppMode::new_session_load(app);
                }
                KeyCode::Char('S') => {
                    info!("Saving sessions mode");
                    app.mode = AppMode::new_session_save();
                }

                KeyCode::Char('+') => {
                    app.pane_manager.resize_pane(&CardinalDirection::Right, 1);
                }
                KeyCode::Char('-') => {
                    app.pane_manager.resize_pane(&CardinalDirection::Left, -1);
                }
                KeyCode::Char('>') => {
                    app.pane_manager.resize_pane(&CardinalDirection::Down, 1);
                }
                KeyCode::Char('<') => {
                    app.pane_manager.resize_pane(&CardinalDirection::Up, -1);
                }
                _ => {}
            }
        }
    }
    Ok(())
}