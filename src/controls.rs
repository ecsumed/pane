use crate::logging::{debug, info, warn};
use crate::session::{load_session, save_session};
use crate::{app::AppControl, command::CommandControl, mode::InputMode, App};
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::Direction;
use std::io;
use tracing::error;
use tui_input::backend::crossterm::EventHandler;

pub async fn handle_event(app: &mut App, event: Event) -> io::Result<()> {
    match app.input_state.mode {
        InputMode::Normal => handle_normal_mode_keys(app, event).await?,
        InputMode::Editing => handle_editing_mode_keys(app, event).await?,
    }
    Ok(())
}

async fn handle_normal_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let Event::Key(key_event) = event {
        if key_event.kind == event::KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => app.exit(),
                KeyCode::Char('h') => {
                    app.pane_manager.split_pane(Direction::Horizontal);
                }
                KeyCode::Char('v') => {
                    app.pane_manager.split_pane(Direction::Vertical);
                }
                KeyCode::Char('c') => {
                    app.input_state.mode = InputMode::Editing;
                }
                KeyCode::Tab => {
                    app.pane_manager.cycle_panes();
                }
                KeyCode::Char('x') => {
                    let id = app.pane_manager.get_active_pane_id();
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
                    let id = app.pane_manager.get_active_pane_id();
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SendControl(id, CommandControl::Execute))
                        .await
                    {
                        warn!("Failed to send AppControl::SendControl: {}", e);
                    }
                }
                KeyCode::Char('p') => {
                    let id = app.pane_manager.get_active_pane_id();
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SendControl(id, CommandControl::Pause))
                        .await
                    {
                        warn!("Failed to send AppControl::SendControl: {}", e);
                    }
                }
                KeyCode::Char('r') => {
                    let id = app.pane_manager.get_active_pane_id();
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SendControl(id, CommandControl::Resume))
                        .await
                    {
                        warn!("Failed to send AppControl::SendControl: {}", e);
                    }
                }
                KeyCode::Char('i') => {
                    let id = app.pane_manager.get_active_pane_id();
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
                    let id = app.pane_manager.get_active_pane_id();
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
                    if let Err(e) = load_session(app) {
                        error!("Error loading session: {}", e);
                    } else {
                        info!("Session loaded successfully!");
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

async fn handle_editing_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    let input = &mut app.input_state.input;
    if let Event::Key(key_event) = event {
        // We only care about press events for keybinds.
        if key_event.kind == event::KeyEventKind::Press {
            match key_event.code {
                KeyCode::Enter => {
                    let id = app.pane_manager.get_active_pane_id();
                    let exec = input.value().to_string();
                    if let Err(e) = app
                        .app_control_tx
                        .send(AppControl::SetCommand(id, exec))
                        .await
                    {
                        warn!("Failed to send AppControl::SetCommand: {}", e);
                    }
                    app.input_state.mode = InputMode::Normal;
                    input.reset();
                }
                KeyCode::Esc => {
                    app.input_state.mode = InputMode::Normal;
                    input.reset();
                }
                _ => {
                    input.handle_event(&event);
                }
            }
        } else {
            // Forward other key events (e.g., release, repeat) to the input handler.
            input.handle_event(&event);
        }
    } else {
        // Forward other events (e.g., mouse) to the input handler.
        input.handle_event(&event);
    }
    Ok(())
}
