use crate::app::{App, AppControl};
use crate::mode::AppMode;
use crate::session::load_session_by_name;
use crossterm::event::{self, Event, KeyCode};
use std::io;
use tracing::error;
use ratatui::widgets::ListState;

use crate::logging::{debug, info, warn};

pub async fn handle_session_load_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let AppMode::SessionLoad { items, state } = &mut app.mode {
        if let Event::Key(key_event) = event {
            if key_event.kind == event::KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Up => {
                        if let Some(selected) = state.selected() {
                            let next = if selected == 0 { items.len() - 1 } else { selected - 1 };
                            state.select(Some(next));
                        }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = state.selected() {
                            let next = (selected + 1) % items.len();
                            state.select(Some(next));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = state.selected() {
                            let session_filename = items[selected].clone();

                            info!("Loading session: {}", session_filename);

                            if let Err(e) = load_session_by_name(app, &session_filename) {
                                error!("Error loading session: {}", e);
                            } else {
                                info!("Session loaded successfully!");
                            }
                        }
                        app.mode = AppMode::Normal;
                    }
                    KeyCode::Esc => {
                        app.mode = AppMode::Normal;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}