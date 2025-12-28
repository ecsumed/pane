use std::collections::HashMap;
use std::time::Duration;

use crokey::{key, KeyCombination};
use ratatui::style::{Color, Modifier, Style};

use super::utils::{app_name, default_sessions_dir_path, default_snapshot_dir_path};
use super::AppConfig;
use crate::config::theme::{Palette, Theme};
use crate::config::utils::default_logging_dir_path;
use crate::controls::KeyMode;
use crate::controls::actions::Action;
use crate::ui::DisplayType;

// GENERAL SETTINGS
const BEEP: bool = false;
const DEFAULT_DISPLAY: DisplayType = DisplayType::RawText;
const EXIT_ON_CHANGE: bool = false;
const EXIT_ON_ERROR: bool = false;
const INTERVAL_SECS: u64 = 5;
const LOG_LEVEL: Option<String> = None;
const MAX_HISTORY: usize = 10;
const WRAP: bool = true;
const ZEN: bool = false;

// THEME
const COLLAPSE_BORDERS: bool = false;

pub fn default_keybindings() -> HashMap<KeyMode, HashMap<KeyCombination, Action>> {
    let mut map = HashMap::new();

    // GLOBAL MODE BINDINGS
    map.insert(KeyMode::Global, HashMap::from([
        (key!(enter), Action::Confirm),
        (key!(tab), Action::Cycle),
        (key!(esc), Action::Escape),
        (key!(q), Action::Quit),
        (key!(down), Action::MoveDown),
        (key!(left), Action::MoveLeft),
        (key!(right), Action::MoveRight),
        (key!(up), Action::MoveUp),
    ]));

    // NORMAL MODE BINDINGS
    map.insert(KeyMode::Normal, HashMap::from([
        (key!(c), Action::EnterCmdMode),
        (key!(shift - d), Action::EnterDisplaySelectMode),
        (key!('?'), Action::EnterHelpMode),
        (key!(o), Action::EnterObserveMode),
        (key!(shift - l), Action::EnterSessionLoadMode),
        (key!(shift - s), Action::EnterSessionSaveMode),
        (key!(d), Action::IntervalDecrease),
        (key!(i), Action::IntervalIncrease),
        (key!(x), Action::KillPane),
        (key!(l), Action::LoadLatestSession),
        (key!('<'), Action::PaneDecreaseHorizontal),
        (key!('-'), Action::PaneDecreaseVertical),
        (key!('>'), Action::PaneIncreaseHorizontal),
        (key!('+'), Action::PaneIncreaseVertical),
        (key!(tab), Action::Cycle),
        (key!(p), Action::Pause),
        (key!(r), Action::Resume),
        (key!(s), Action::SaveSession),
        (key!(h), Action::SplitHorizontal),
        (key!(v), Action::SplitVertical),
        (key!(w), Action::WrapToggle),
        (key!(z), Action::ZenToggle),
    ]));

    // CMD EDIT MODE BINDINGS
    map.insert(KeyMode::CmdEdit, HashMap::from([
        (key!(tab), Action::TabComplete),
    ]));

    // SESSION LOAD BINDINGS
    map.insert(KeyMode::SessionLoad, HashMap::new());

    // SESSION SAVE MODE BINDINGS
    map.insert(KeyMode::SessionSave, HashMap::new());

    // DISPLAY SELECT BINDINGS
    map.insert(KeyMode::DisplayTypeSelect, HashMap::new());

    // HELP BINDINGS
    map.insert(KeyMode::Help, HashMap::new());

    // OBSERVE BINDINGS
    map.insert(KeyMode::Observe, HashMap::new());

    map
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            border_active: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            border_inactive: Style::default().fg(Color::DarkGray),
            border_label: Style::default().fg(Color::Reset).bg(Color::Rgb(40, 40, 40)),
            meta_label: Style::default().fg(Color::DarkGray),
            meta_value: Style::default().fg(Color::White),
            meta_highlight: Style::default().fg(Color::Yellow).bold(),
            meta_secondary: Style::default().fg(Color::Blue),  
            diff_add: Style::default().fg(Color::Green).bg(Color::Rgb(20, 40, 20)),
            diff_remove: Style::default().fg(Color::Red).bg(Color::Rgb(40, 20, 20)),
            search_match: Style::default().fg(Color::Black).bg(Color::Yellow),
            multiline_timestamp: Style::default().fg(Color::LightGreen).bold(),
            output: Style::default().fg(Color::Gray),
        }
    }
}
impl Default for Theme {
    fn default() -> Self {
        Theme {
            collapse_borders: COLLAPSE_BORDERS,
            palette: Palette::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        use directories::ProjectDirs;

        let proj_dirs = ProjectDirs::from("io", app_name(), app_name());

        AppConfig {
            interval: Duration::from_secs(INTERVAL_SECS),
            zen: ZEN,
            beep: BEEP,
            err_exit: EXIT_ON_ERROR,
            chg_exit: EXIT_ON_CHANGE,
            wrap: WRAP,
            default_display: DEFAULT_DISPLAY,
            log_level: LOG_LEVEL,
            max_history: MAX_HISTORY,
            logs_dir: default_logging_dir_path(&proj_dirs),
            sessions_dir: default_sessions_dir_path(&proj_dirs),
            snapshot_dir: default_snapshot_dir_path(&proj_dirs),
            keybindings: default_keybindings(),
            theme: Theme::default(),
        }
    }
}