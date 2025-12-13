use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use crokey::KeyCombination;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use super::utils::{app_name, deserialize_duration, get_home_dir};
use crate::controls::actions::Action;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub interval: Duration,
    pub sessions_dir: PathBuf,
    pub snapshot_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub log_level: Option<String>,
    pub keybindings: HashMap<KeyCombination, Action>,
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Configuration loaded successfully:")?;
        writeln!(f, "  Interval: {:?}", self.interval)?;
        writeln!(
            f,
            "  Log Level: {}",
            self.log_level.as_deref().unwrap_or("N/A")
        )?;
        writeln!(f, "  Logs Directory: {:?}", self.logs_dir)?;
        writeln!(f, "  Sessions Directory: {:?}", self.sessions_dir)?;
        writeln!(f, "  Snapshot Directory: {:?}", self.snapshot_dir)?;

        writeln!(f, "  Keys:")?;

        for (combination, action) in self.keybindings.iter() {
            let key_str = format!("{:?}", combination.codes);
            writeln!(f, "    Key: {:<12} -> Action: {:?}", key_str, action)?;
        }
        Ok(())
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let app_name_str = app_name();

        let mut builder = config::Config::builder();

        let default_config = AppConfig::default();
        builder = builder
            .set_default(
                "interval",
                default_config.interval.as_secs().to_string() + "s",
            )?
            .set_default("log_level", default_config.log_level)?
            .set_default("logs_dir", default_config.logs_dir.to_str().unwrap_or(""))?
            .set_default(
                "sessions_dir",
                default_config.sessions_dir.to_str().unwrap_or(""),
            )?
            .set_default(
                "snapshot_dir",
                default_config.snapshot_dir.to_str().unwrap_or(""),
            )?;

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

        builder = builder.add_source(config::File::from(config_path).required(false));

        builder = builder.add_source(
            config::Environment::with_prefix(&app_name_str.to_uppercase().replace('-', "__"))
                .separator("__"),
        );

        builder.build()?.try_deserialize()
    }
}
