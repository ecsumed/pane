use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use crokey::KeyCombination;
use directories::ProjectDirs;
use figment::{
    Figment, providers::{Env, Format, Serialized, Toml}
};
use serde::{Deserialize, Serialize};

use crate::logging::{debug, info};
use super::utils::{app_name, deserialize_duration, get_home_dir};
use crate::controls::{KeyMode, actions::Action};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub interval: Duration,
    pub beep: bool,
    pub err_exit: bool,
    pub chg_exit: bool,
    pub wrap: bool,
    pub max_history: usize,
    pub zen: bool,
    pub sessions_dir: PathBuf,
    pub snapshot_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub log_level: Option<String>,
    pub keybindings: HashMap<KeyMode, HashMap<KeyCombination, Action>>,
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Configuration loaded successfully:")?;
        writeln!(f, "  Interval: {:?}", self.interval)?;
        writeln!(f, "  Beep: {}", self.beep)?;
        writeln!(f, "  Exit on Error: {}", self.err_exit)?;
        writeln!(f, "  Exit on Change: {}", self.chg_exit)?;
        writeln!(f, "  Wrap: {}", self.wrap)?;
        writeln!(f, "  Max History: {}", self.max_history)?;
        writeln!(f, "  Zen: {}", self.zen)?;
        writeln!(
            f,
            "  Log Level: {}",
            self.log_level.as_deref().unwrap_or("N/A")
        )?;
        writeln!(f, "  Logs Directory: {:?}", self.logs_dir)?;
        writeln!(f, "  Sessions Directory: {:?}", self.sessions_dir)?;
        writeln!(f, "  Snapshot Directory: {:?}", self.snapshot_dir)?;

        for (keymode, bindings) in self.keybindings.iter() {
            writeln!(f, "  Keys {}:", keymode)?;
            for (combination, action) in bindings.iter() {
                let key_str = format!("{:?}", combination.codes);
                writeln!(f, "    Key: {:<12} -> Action: {:?}", key_str, action)?;
            }
        }
        Ok(())
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, figment::Error> {
        let app_name_str = app_name();
        let config_path = AppConfig::get_config_path();
        let env_prefix = format!("{}_", app_name_str.to_uppercase().replace('-', "_"));
    
        Figment::new()
            // Load defaults first
            .merge(Serialized::defaults(AppConfig::default()))
            
            // Load from config.yaml
            .merge(Toml::file(config_path))

            // Load from Environment Variables
            .merge(Env::prefixed(&env_prefix).split("__"))
            
            .extract()
    }

    pub fn merge_cli(&mut self, cli: &crate::cli::Cli) {
        if cli.beep {
            self.beep = true;
        }

        if cli.err_exit {
            self.err_exit = true;
        }

        if cli.chg_exit {
            self.chg_exit = true;
        }

        if cli.zen {
            self.zen = true;
        }

        if cli.no_wrap {
            self.wrap = false;
        }

        if let Some(max_history) = cli.max_history {
            self.max_history = max_history;
        }
        
        if let Some(interval) = cli.interval {
            self.interval = Duration::from_secs(interval);
        }

        if cli.verbose.is_present() {
            let level = cli.verbose.log_level_filter().to_string();
            self.log_level = Some(level.to_lowercase());
        }
    }

    fn get_config_path() -> PathBuf {
        let config_path = {
            #[cfg(target_os = "macos")]
            if let Some(home_dir) = get_home_dir() {
                home_dir
                    .join(".config")
                    .join(app_name())
                    .join("config.toml")
            } else {
                let proj_dirs = ProjectDirs::from("io", app_name(), app_name());
                proj_dirs
                    .map(|p| p.config_dir().join("config.toml"))
                    .unwrap_or_default()
            }
            #[cfg(not(target_os = "macos"))]
            {
                let proj_dirs = ProjectDirs::from("io", app_name(), app_name());
                proj_dirs
                    .map(|p| p.config_dir().join("config.toml"))
                    .unwrap_or_default()
            }
        };

        config_path
    }
}
