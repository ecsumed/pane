
use std::io;

use crossterm::event::{self, Event};
use tui_input::backend::crossterm::EventHandler;

use crate::app::App;
use crate::controls::actions::Action;
use crate::logging::debug;
use crate::mode::{AppMode, DiffMode};

pub async fn handle_observe_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let AppMode::Observe { 
        selected_history_idx, 
        diff_mode,
        search_input,
        ..
    } = &mut app.mode {
        if let Event::Key(key_event) = event {
            if key_event.kind == event::KeyEventKind::Press {
                if let Some(key_combination) = app.combiner.transform(key_event) {
                    if let Some(action) = app.config.keybindings.get(&key_combination) {
                        match action {
                            Action::Escape | Action::Quit => {
                                app.mode = AppMode::Normal;
                            }
                            Action::MoveUp => {
                                if *selected_history_idx > 0 {
                                    *selected_history_idx -= 1;
                                }
                            }
                            Action::MoveDown => {
                                let active_id = app.pane_manager.active_pane_id;
                                let cmd = app.tasks.get(&active_id).unwrap();

                                if *selected_history_idx < cmd.output_history.len().saturating_sub(1) {
                                    *selected_history_idx += 1;
                                }
                            }
                            Action::Cycle => {
                                *diff_mode = match diff_mode {
                                    DiffMode::None => DiffMode::Line,
                                    DiffMode::Line => DiffMode::Word,
                                    DiffMode::Word => DiffMode::Char,
                                    DiffMode::Char => DiffMode::None,
                                };
                                debug!("Cycling diff to {}", diff_mode);
                            }
                            _ => {
                                search_input.handle_event(&event);
                            }
                        } 
                    } else {
                        search_input.handle_event(&event);
                    }
                }
            }
        }
    }
    Ok(())
}
