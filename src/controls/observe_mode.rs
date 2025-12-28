use std::io;

use crokey::KeyCombination;
use crossterm::event::{self, Event};
use tui_input::backend::crossterm::EventHandler;

use crate::app::App;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::logging::debug;
use crate::mode::{AppMode, DiffMode, ObserveFocus};

pub async fn handle_observe_mode_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context: KeyMode = app.mode.key_mode();

    let AppMode::Observe {
        selected_history_idx,
        diff_mode,
        search_input,
        focus,
        scroll_offset,
        ..
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

    if let Some(act) = action {
        match act {
            Action::Escape | Action::Quit => {
                app.mode = AppMode::Normal;
            }

            Action::MoveLeft => *focus = ObserveFocus::Content,
            Action::MoveRight => *focus = ObserveFocus::History,
            Action::Search => *focus = ObserveFocus::Search,

            Action::MoveUp => match focus {
                ObserveFocus::History => {
                    if *selected_history_idx > 0 {
                        *selected_history_idx -= 1;
                        *scroll_offset = 0; // Reset scroll when changing history
                    }
                }
                ObserveFocus::Content => {
                    *scroll_offset = scroll_offset.saturating_sub(1);
                }
                ObserveFocus::Search => {}
            },

            Action::MoveDown => match focus {
                ObserveFocus::History => {
                    let active_id = app.pane_manager.active_pane_id;
                    let cmd = app.tasks.get(&active_id).unwrap();

                    if *selected_history_idx < cmd.output_history.len().saturating_sub(1) {
                        *selected_history_idx += 1;
                        *scroll_offset = 0;
                    }
                }
                ObserveFocus::Content => {
                    *scroll_offset = scroll_offset.saturating_add(1);
                }
                ObserveFocus::Search => {}
            },

            Action::WrapToggle => match focus {
                ObserveFocus::History => {}
                ObserveFocus::Content => {
                    app.config.wrap = !app.config.wrap;
                }
                ObserveFocus::Search => {}
            },

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
                if matches!(focus, ObserveFocus::Search) {
                    search_input.handle_event(&event);
                }
            }
        }
    } else {
        search_input.handle_event(&event);
    }
    Ok(())
}
