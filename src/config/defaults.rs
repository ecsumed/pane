use std::collections::HashMap;
use std::time::Duration;

use crate::config::utils::default_logging_dir_path;
use crate::controls::actions::Action;
use crokey::{key, KeyCombination};

use super::utils::{default_snapshot_dir_path, default_sessions_dir_path};

use super::AppConfig;
use super::utils::app_name;

pub const DEFAULT_INTERVAL_SECS: u64 = 5;
pub const LOG_LEVEL: Option<String> = None;

pub fn default_keybindings() -> HashMap<KeyCombination, Action> {
    HashMap::from([
        (key!(enter), Action::Confirm),
        (key!(tab), Action::CyclePanes),
        (key!(c), Action::EnterCmdMode),
        (key!(shift - d), Action::EnterDisplaySelectMode),
        (key!(shift - l), Action::EnterSessionLoadMode),
        (key!(shift - s), Action::EnterSessionSaveMode),
        (key!(esc), Action::Escape),
        (key!(d), Action::IntervalDecrease),
        (key!(i), Action::IntervalIncrease),
        (key!(x), Action::KillPane),
        (key!(l), Action::LoadLatestSession),
        (key!(down), Action::MoveDown),
        (key!(left), Action::MoveLeft),
        (key!(right), Action::MoveRight),
        (key!(up), Action::MoveUp),
        (key!(shift - '<'), Action::PaneDecreaseHorizontal),
        (key!('-'), Action::PaneDecreaseVertical),
        (key!(shift - '>'), Action::PaneIncreaseHorizontal),
        (key!(shift - '+'), Action::PaneIncreaseVertical),
        (key!(p), Action::Pause),
        (key!(q), Action::Quit),
        (key!(r), Action::Resume),
        (key!(s), Action::SaveSession),
        (key!(h), Action::SplitHorizontal),
        (key!(v), Action::SplitVertical),
        (key!(tab), Action::TabComplete),
    ])
}

impl Default for AppConfig {
    fn default() -> Self {
        use directories::ProjectDirs;

        let proj_dirs = ProjectDirs::from("io", app_name(), app_name());

        AppConfig {
            interval: Duration::from_secs(DEFAULT_INTERVAL_SECS),
            log_level: LOG_LEVEL,
            logs_dir: default_logging_dir_path(&proj_dirs),
            sessions_dir: default_sessions_dir_path(&proj_dirs),
            snapshot_dir: default_snapshot_dir_path(&proj_dirs),
            keybindings:default_keybindings(),
        }
    }
}
