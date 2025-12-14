use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;

use crate::app::App;
use crate::controls::KeyMode;
use crate::controls::actions::Action;
use crate::mode::AppMode;

pub async fn handle_help_keys(app: &mut App, event: Event) -> io::Result<()> {
    let Event::Key(key_event) = event else {
        return Ok(());
    };
    if key_event.kind != event::KeyEventKind::Press {
        return Ok(());
    }

    let key_comb: KeyCombination = KeyCombination::from(key_event);

    let current_context = app.mode.key_mode();

    let action = app.config.keybindings
        .get(&current_context)
        .and_then(|map| map.get(&key_comb))
        .or_else(|| {
            app.config.keybindings
                .get(&KeyMode::Global)
                .and_then(|map| map.get(&key_comb))
        });
    
    if let Some(act) = action {
        match act {
            Action::EnterHelpMode | Action::Escape | Action::Quit => {
                app.mode = AppMode::Normal;
            }
            _ => (),
        }
    }
    Ok(())
}