use std::collections::HashMap;
use std::time::Duration;

use crokey::{key, KeyCombination};
use ratatui::style::{Color, Modifier, Style};

use super::utils::{app_name, default_sessions_dir_path};
use super::AppConfig;
use crate::config::theme::{Palette, Theme, ThemeMode};
use crate::config::utils::default_logging_dir_path;
use crate::controls::actions::Action;
use crate::controls::KeyMode;
use crate::ui::DisplayType;

// GENERAL SETTINGS
const BEEP: bool = false;
const DEFAULT_DISPLAY: DisplayType = DisplayType::RawText;
const EXIT_ON_CHANGE: bool = false;
const EXIT_ON_ERROR: bool = false;
const INTERVAL_SECS: u64 = 5;
const TIMEOUT_SECS: u64 = 60;
const LOG_LEVEL: Option<String> = None;
const MAX_HISTORY: usize = 10;
const WRAP: bool = true;
const ZEN: bool = false;

// THEME
const COLLAPSE_BORDERS: bool = false;
const MODE: ThemeMode = ThemeMode::System;
const SHOW_STATE: bool = true;
const SHOW_LAST_UPDATED: bool = true;
const SHOW_DISPLAY_TYPE: bool = true;
const SHOW_INLINE_DELETIONS: bool = false;
const SHOW_HISTORY_METER: bool = true;
const SHOW_STATUS_BAR: bool = true;

pub fn default_keybindings() -> HashMap<KeyMode, HashMap<KeyCombination, Action>> {
    let mut map = HashMap::new();

    // GLOBAL MODE BINDINGS
    map.insert(
        KeyMode::Global,
        HashMap::from([
            (key!(enter), Action::Confirm),
            (key!(tab), Action::Cycle),
            (key!(esc), Action::Escape),
            (key!(q), Action::Quit),
            (key!(down), Action::MoveDown),
            (key!(left), Action::MoveLeft),
            (key!(right), Action::MoveRight),
            (key!(up), Action::MoveUp),
        ]),
    );

    // NORMAL MODE BINDINGS
    map.insert(
        KeyMode::Normal,
        HashMap::from([
            (key!(c), Action::EnterCmdMode),
            (key!(shift - d), Action::EnterDisplaySelectMode),
            (key!('?'), Action::EnterHelpMode),
            (key!(o), Action::EnterObserveMode),
            (key!(shift - l), Action::EnterSessionLoadMode),
            (key!(shift - s), Action::EnterSessionSaveMode),
            (key!(space), Action::Execute),
            (key!(d), Action::IntervalDecrease),
            (key!(i), Action::IntervalIncrease),
            (key!(x), Action::KillPane),
            (key!(l), Action::LoadLatestSession),
            (key!('<'), Action::ResizeLeft),
            (key!('-'), Action::ShrinkHeight),
            (key!('>'), Action::ResizeRight),
            (key!('+'), Action::GrowHeight),
            (key!('='), Action::EqualizePanes),
            (key!(shift - left), Action::ResizeLeft),
            (key!(shift - right), Action::ResizeRight),
            (key!(shift - up), Action::ResizeUp),
            (key!(shift - down), Action::ResizeDown),
            (key!(tab), Action::Cycle),
            (key!(p), Action::Pause),
            (key!(r), Action::Resume),
            (key!(s), Action::SaveSession),
            (key!(h), Action::SplitHorizontal),
            (key!(v), Action::SplitVertical),
            (key!(w), Action::WrapToggle),
            (key!(z), Action::ZenToggle),
        ]),
    );

    // CMD EDIT MODE BINDINGS
    map.insert(
        KeyMode::CmdEdit,
        HashMap::from([(key!(tab), Action::TabComplete)]),
    );

    // SESSION LOAD BINDINGS
    map.insert(
        KeyMode::SessionLoad,
        HashMap::from([
            (key!(ctrl - d), Action::Delete),
            (key!(ctrl - r), Action::Rename),
        ]),
    );

    // SESSION SAVE MODE BINDINGS
    map.insert(KeyMode::SessionSave, HashMap::new());

    // DISPLAY SELECT BINDINGS
    map.insert(KeyMode::DisplayTypeSelect, HashMap::new());

    // HELP BINDINGS
    map.insert(
        KeyMode::Help,
        HashMap::from([
            (key!('/'), Action::Search),
            (key!(g), Action::ScrollTop),
            (key!(shift - g), Action::ScrollBottom),
        ]),
    );

    // OBSERVE BINDINGS
    map.insert(
        KeyMode::Observe,
        HashMap::from([
            (key!('/'), Action::Search),
            (key!(n), Action::NextMatch),
            (key!(shift - n), Action::PrevMatch),
            (key!(w), Action::WrapToggle),
            (key!(g), Action::ScrollTop),
            (key!(shift - g), Action::ScrollBottom),
        ]),
    );

    map
}

impl Palette {
    pub fn dark_preset() -> Self {
        Self {
            border_active: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            border_inactive: Style::default().fg(Color::DarkGray),
            border_label: Style::default().fg(Color::Reset).bg(Color::Rgb(40, 40, 40)),
            chart_bar: Style::default().fg(Color::LightBlue),
            chart_line: Style::default().fg(Color::LightMagenta),
            chart_scatter: Style::default().fg(Color::LightGreen),
            counter_key: Style::default().fg(Color::LightGreen).bold(),
            diff_add: Style::default().fg(Color::Green).bg(Color::Rgb(20, 40, 20)),
            diff_remove: Style::default().fg(Color::Red).bg(Color::Rgb(40, 20, 20)),
            error: Style::default().fg(Color::Red),
            h1: Style::default().fg(Color::Red).bold(),
            h2: Style::default().fg(Color::LightRed).bold(),
            meta_highlight: Style::default().fg(Color::Yellow).bold(),
            meta_label: Style::default().fg(Color::DarkGray),
            meta_meter: Style::default().fg(Color::LightRed),
            meta_secondary: Style::default().fg(Color::Blue),
            meta_value: Style::default().fg(Color::White),
            multiline_timestamp: Style::default().fg(Color::LightGreen).bold(),
            output: Style::default().fg(Color::Gray),
            scroll_bar: Style::default().fg(Color::LightRed),
            scroll_track: Style::default().fg(Color::Blue),
            search_match: Style::default().fg(Color::Black).bg(Color::Yellow),
            spark_line: Style::default().fg(Color::LightBlue),
        }
    }

    pub fn light_preset() -> Self {
        Self {
            border_active: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            border_inactive: Style::default().fg(Color::Gray),
            border_label: Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(230, 230, 230)),
            chart_bar: Style::default().fg(Color::Blue),
            chart_line: Style::default().fg(Color::Magenta),
            chart_scatter: Style::default().fg(Color::Green),
            counter_key: Style::default().fg(Color::DarkGray).bold(),
            diff_add: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(220, 255, 220)),
            diff_remove: Style::default()
                .fg(Color::Red)
                .bg(Color::Rgb(255, 220, 220)),
            error: Style::default().fg(Color::Red),
            h1: Style::default().fg(Color::Red).bold(),
            h2: Style::default().fg(Color::Rgb(150, 0, 0)).bold(),
            meta_highlight: Style::default().fg(Color::Rgb(100, 100, 0)).bold(),
            meta_label: Style::default().fg(Color::Gray),
            meta_meter: Style::default().fg(Color::Red),
            meta_secondary: Style::default().fg(Color::Blue),
            meta_value: Style::default().fg(Color::Black),
            multiline_timestamp: Style::default().fg(Color::DarkGray).bold(),
            output: Style::default().fg(Color::Black),
            scroll_bar: Style::default().fg(Color::Red),
            scroll_track: Style::default().fg(Color::LightBlue),
            search_match: Style::default().fg(Color::Black).bg(Color::Yellow),
            spark_line: Style::default().fg(Color::Blue),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            collapse_borders: COLLAPSE_BORDERS,
            dark_palette: Palette::dark_preset(),
            light_palette: Palette::light_preset(),
            palette: Palette::dark_preset(),
            mode: MODE,
            show_state: SHOW_STATE,
            show_last_updated: SHOW_LAST_UPDATED,
            show_display_type: SHOW_DISPLAY_TYPE,
            show_history_meter: SHOW_HISTORY_METER,
            show_inline_deletions: SHOW_INLINE_DELETIONS,
            show_status_bar: SHOW_STATUS_BAR,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        use directories::ProjectDirs;

        let proj_dirs = ProjectDirs::from("io", app_name(), app_name());

        AppConfig {
            interval: Duration::from_secs(INTERVAL_SECS),
            timeout: Duration::from_secs(TIMEOUT_SECS),
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
            keybindings: default_keybindings(),
            theme: Theme::default(),
        }
    }
}
