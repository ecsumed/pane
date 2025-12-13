use std::io;

use crokey::crossterm::event::{self, Event};
use crokey::KeyCombination;

use crate::app::App;
use crate::controls::actions::Action;
use crate::mode::AppMode;

pub async fn handle_help_keys(app: &mut App, event: Event) -> io::Result<()> {
    if let Event::Key(key_event) = event {
        if key_event.kind == event::KeyEventKind::Press {
            let key_combination: KeyCombination = KeyCombination::from(key_event);
            if let Some(action) = app.config.keybindings.get(&key_combination) {
                match action {
                    Action::EnterHelpMode | Action::Escape | Action::Quit => {
                        app.mode = AppMode::Normal;
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(())
}