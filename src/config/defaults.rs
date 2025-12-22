use std::collections::HashMap;
use std::time::Duration;

use crokey::{key, KeyCombination};

use super::utils::{app_name, default_sessions_dir_path, default_snapshot_dir_path};
use super::AppConfig;
use crate::config::utils::default_logging_dir_path;
use crate::controls::KeyMode;
use crate::controls::actions::Action;

const INTERVAL_SECS: u64 = 5;
const BEEP: bool = false;
const MAX_HISTORY: usize = 1000;
const LOG_LEVEL: Option<String> = None;

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

impl Default for AppConfig {
    fn default() -> Self {
        use directories::ProjectDirs;

        let proj_dirs = ProjectDirs::from("io", app_name(), app_name());

        AppConfig {
            interval: Duration::from_secs(INTERVAL_SECS),
            beep: BEEP,
            log_level: LOG_LEVEL,
            max_history: MAX_HISTORY,
            logs_dir: default_logging_dir_path(&proj_dirs),
            sessions_dir: default_sessions_dir_path(&proj_dirs),
            snapshot_dir: default_snapshot_dir_path(&proj_dirs),
            keybindings: default_keybindings(),
        }
    }
}
