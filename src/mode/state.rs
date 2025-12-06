use ratatui::widgets::ListState;
use tui_input::Input;

use crate::app::{self, App};
use crate::session;

#[derive(Debug, Default)]
pub enum AppMode {
    #[default]
    Normal,
    CmdEdit {
        input: Input,
    },
    SessionLoad {
        state: ListState, 
        items: Vec<String>,
    },
    SessionSave {
        input: Input,
    },
}

impl AppMode {
    pub fn new() -> Self {
        AppMode::Normal
    }

    pub fn new_cmd_edit() -> Self {
        AppMode::CmdEdit {
            input: Input::default(),
        }
    }

    pub fn new_session_load(app: &App) -> Self {
        let sessions = session::fetch_session_filenames(&app.config
        
        ).unwrap();

        let mut state = ListState::default();
        if !sessions.is_empty() {
            state.select(Some(0));
        }

        AppMode::SessionLoad {
            items: sessions,
            state,
        }
    }

    pub fn new_session_save() -> Self {
        AppMode::SessionSave {
            input: Input::default(),
        }
    }

    // pub fn new_observing() -> Self {
    //     AppMode::Observing
    // }
}