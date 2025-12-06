use crate::app::{App, AppControl};
use crate::mode::AppMode;
use crossterm::event::{self, Event, KeyCode};
use std::io;

use crate::logging::{debug, info, warn};

pub async fn handle_display_type_select_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let AppMode::DisplayTypeSelect { items, state } = &mut app.mode {
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
                            let display_type = items[selected].clone();
                            let id = app.pane_manager.active_pane_id;

                            info!("Changing to display: {:?}", display_type);
                            if let Err(e) = app
                                .app_control_tx
                                .send(AppControl::SetDisplay(id, display_type))
                                .await
                            {
                                warn!("Failed to send AppControl::SetDisplay: {}", e);
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
