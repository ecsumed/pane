use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;

use crate::app::App;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::mode::AppMode;

pub async fn handle_help_keys(app: &mut App, event: Event) -> io::Result<()> {
    let current_context = app.mode.key_mode();

    let AppMode::Help {
        scroll_offset,
        max_scroll,
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
            Action::EnterHelpMode | Action::Escape | Action::Quit => {
                app.mode = AppMode::Normal;
            }

            Action::MoveUp => {
                *scroll_offset = scroll_offset.saturating_sub(1);
            }

            Action::MoveDown => {
                *scroll_offset = (*scroll_offset).saturating_add(1).min(*max_scroll);
            }

            Action::ScrollTop => {
                *scroll_offset = 0;
            }

            Action::ScrollBottom => {
                *scroll_offset = *max_scroll;
            }
            _ => (),
        }
    }
    Ok(())
}
