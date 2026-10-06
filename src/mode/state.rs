use std::collections::HashMap;
use std::fmt;

use ratatui::widgets::{ListState, ScrollbarState};
use strum::IntoEnumIterator;
use tui_input::Input;

use crate::app::App;
use crate::config::AppConfig;
use crate::controls::KeyMode;
use crate::logging::error;
use crate::mode::Picker;
use crate::pane::PaneKey;
use crate::session::{self, SessionEntry, SessionPreview};
use crate::shell_history::ShellHistoryManager;
use crate::ui::DisplayType;

#[derive(Debug, Default)]
pub enum AppMode {
    #[default]
    Normal,
    CmdEdit {
        input: Input,
        state: ListState,
        suggestions: Vec<String>,
        history: ShellHistoryManager,
    },
    SessionLoad {
        picker: Picker<SessionEntry>,
        prompt: SessionPrompt,
        previews: HashMap<String, Result<SessionPreview, String>>,
    },
    SessionSave {
        input: Input,
    },
    DisplayTypeSelect {
        picker: Picker<DisplayType>,
    },
    Help {
        scroll_offset: u16,
        max_scroll: u16,
        scrollbar_state: ScrollbarState,
    },
    Observe {
        active_id: PaneKey,
        selected_history_idx: usize,
        last_history_len: usize,
        diff_mode: DiffMode,
        search_input: Input,
        history_list_state: ListState,
        focus: ObserveFocus,
        scroll_offset: u16,
        max_scroll: u16,
        scrollbar_state: ScrollbarState,
    },
}

#[derive(Debug, Default)]
pub enum SessionPrompt {
    #[default]
    None,
    Error(String),
    ConfirmDelete,
    Rename(Input),
}

#[derive(Debug, Default, PartialEq)]
pub enum ObserveFocus {
    Content,
    #[default]
    History,
    Search,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum DiffMode {
    None,
    Line,
    #[default]
    Word,
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
            AppMode::Help { .. } => "Help",
            AppMode::Observe { .. } => "Observe",
        };
        write!(f, "{}", name)
    }
}

impl AppMode {
    pub fn key_mode(&self) -> KeyMode {
        match self {
            AppMode::Normal => KeyMode::Normal,
            AppMode::CmdEdit { .. } => KeyMode::CmdEdit,
            AppMode::SessionLoad { .. } => KeyMode::SessionLoad,
            AppMode::SessionSave { .. } => KeyMode::SessionSave,
            AppMode::DisplayTypeSelect { .. } => KeyMode::DisplayTypeSelect,
            AppMode::Observe { .. } => KeyMode::Observe,
            AppMode::Help { .. } => KeyMode::Help,
        }
    }

    pub fn new_cmd_edit(old_input: Option<String>) -> Self {
        let input = match old_input {
            Some(val) => Input::default().with_value(val),
            None => Input::default(),
        };

        AppMode::CmdEdit {
            input,
            state: ListState::default(),
            suggestions: Vec::new(),
            history: ShellHistoryManager::new(),
        }
    }

    pub fn new_session_load(app: &App) -> Self {
        let sessions = session::fetch_sessions(&app.config).unwrap_or_else(|e| {
            error!("Failed to read sessions: {}", e);
            Vec::new()
        });

        let mut mode = AppMode::SessionLoad {
            picker: Picker::new(sessions, |s: &SessionEntry| s.name.clone()),
            prompt: SessionPrompt::None,
            previews: HashMap::new(),
        };
        mode.load_session_preview(&app.config);
        mode
    }

    pub fn load_session_preview(&mut self, config: &AppConfig) {
        let AppMode::SessionLoad {
            picker, previews, ..
        } = self
        else {
            return;
        };
        if let Some(entry) = picker.selected() {
            previews.entry(entry.file_name.clone()).or_insert_with(|| {
                session::load_session_preview(config, &entry.file_name).map_err(|e| e.to_string())
            });
        }
    }

    pub fn new_session_save() -> Self {
        AppMode::SessionSave {
            input: Input::default(),
        }
    }

    pub fn new_display_type_select(current: Option<DisplayType>) -> Self {
        let mut picker = Picker::new(DisplayType::iter().collect(), |dt: &DisplayType| {
            format!("{} {}", dt.group(), dt.label())
        });
        if let Some(index) = current.and_then(|c| picker.items().iter().position(|&dt| dt == c)) {
            picker.select_item(index);
        }

        AppMode::DisplayTypeSelect { picker }
    }

    pub fn new_help() -> Self {
        AppMode::Help {
            scroll_offset: 0,
            max_scroll: 0,
            scrollbar_state: ScrollbarState::default(),
        }
    }

    pub fn _scroll_bottom(&mut self) {
        if let AppMode::Help {
            scroll_offset,
            max_scroll,
            ..
        } = self
        {
            *scroll_offset = *max_scroll;
        }
    }

    pub fn new_observing(app: &App) -> Self {
        let diff_mode = app.observe_diff_mode;
        let active_id = app.pane_manager.active_pane_id;

        AppMode::Observe {
            active_id,
            selected_history_idx: 0,
            last_history_len: 0,
            diff_mode,
            search_input: Input::default(),
            history_list_state: ListState::default(),
            focus: ObserveFocus::default(),
            scroll_offset: 0,
            max_scroll: 0,
            scrollbar_state: ScrollbarState::default(),
        }
    }
}
