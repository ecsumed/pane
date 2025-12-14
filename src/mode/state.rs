use std::fmt;

use ratatui::widgets::ListState;
use strum::IntoEnumIterator;
use tui_input::Input;

use crate::app::App;
use crate::history::HistoryManager;
use crate::pane::PaneKey;
use crate::session;
use crate::ui::DisplayType;

#[derive(Debug, Default)]
pub enum AppMode {
    #[default]
    Normal,
    CmdEdit {
        input: Input,
        state: ListState,
        suggestions: Vec<String>,
        history: HistoryManager,
    },
    SessionLoad {
        state: ListState,
        items: Vec<String>,
    },
    SessionSave {
        input: Input,
    },
    DisplayTypeSelect {
        state: ListState,
        items: Vec<DisplayType>,
    },
    Help,
    Observe {
        active_id: PaneKey,
        selected_history_idx: usize,
        last_history_len: usize,
        diff_mode: DiffMode,
        search_input: Input,
        history_list_state: ListState, 
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum DiffMode {
    None,
    Line,
    Word,
    #[default]
    Char,
}

impl fmt::Display for DiffMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiffMode::None => write!(f, "Plain"),
            DiffMode::Line => write!(f, "Line"),
            DiffMode::Word => write!(f, "Word"),
            DiffMode::Char => write!(f, "Char"),
        }
    }
}

impl fmt::Display for AppMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AppMode::Normal => "Normal",
            AppMode::CmdEdit { .. } => "Command Edit",
            AppMode::SessionLoad { .. } => "Load Session",
            AppMode::SessionSave { .. } => "Save Session",
            AppMode::DisplayTypeSelect { .. } => "Select Display",
            AppMode::Help => "Help",
            AppMode::Observe { .. } => "Observe"
        };
        write!(f, "{}", name)
    }
}

impl AppMode {
    pub fn new_cmd_edit() -> Self {
        AppMode::CmdEdit {
            input: Input::default(),
            state: ListState::default(),
            suggestions: Vec::new(),
            history: HistoryManager::new(),
        }
    }

    pub fn new_session_load(app: &App) -> Self {
        let sessions = session::fetch_session_filenames(&app.config).unwrap();

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

    pub fn new_display_type_select() -> Self {
        let items: Vec<DisplayType> = DisplayType::iter().collect();

        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }

        AppMode::DisplayTypeSelect { items, state }
    }

    pub fn new_observing(app: &App) -> Self {
        let diff_mode = DiffMode::default();
        let active_id = app.pane_manager.active_pane_id;

        AppMode::Observe {
            active_id: active_id,
            selected_history_idx: 0,
            last_history_len: 0,
            diff_mode: diff_mode,
            search_input: Input::default(),
            history_list_state: ListState::default(),
        }
    }
}
