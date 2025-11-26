use directories::ProjectDirs;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

// Function to get the home directory, handling the Option.
// This function must be defined before it is used by other functions.
fn get_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

// Pass the ProjectDirs instance explicitly to avoid repeated calls
fn default_sessions_dir_path(proj_dirs: &Option<ProjectDirs>) -> PathBuf {
    // Check if we're on macOS and a home directory exists
    #[cfg(target_os = "macos")]
    if let Some(home_dir) = get_home_dir() {
        return home_dir.join(".config").join(app_name()).join("sessions");
    }

    // Default behavior for other OS or if home directory is not found
    if let Some(dirs) = proj_dirs {
        dirs.data_dir().join("sessions")
    } else {
        PathBuf::from("./data/sessions")
    }
}

// Pass the ProjectDirs instance explicitly to avoid repeated calls
fn default_snapshot_dir_path(proj_dirs: &Option<ProjectDirs>) -> PathBuf {
    #[cfg(target_os = "macos")]
    if let Some(home_dir) = get_home_dir() {
        return home_dir.join(".config").join(app_name()).join("snapshots");
    }

    if let Some(dirs) = proj_dirs {
        dirs.data_dir().join("snapshots")
    } else {
        PathBuf::from("./data/snapshots")
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub interval: Duration,
    pub sessions_dir: PathBuf,
    pub snapshot_dir: PathBuf,
    pub keys: Keys,
}

impl Default for AppConfig {
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("io", app_name(), app_name());
        AppConfig {
            interval: Duration::from_secs(5),
            sessions_dir: default_sessions_dir_path(&proj_dirs),
            snapshot_dir: default_snapshot_dir_path(&proj_dirs),
            keys: Keys::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Keys {
    pub v: char,
    pub h: char,
    pub p: char,
    pub r: char,
    pub x: char,
}

impl Default for Keys {
    fn default() -> Self {
        Keys {
            v: 'v',
            h: 'h',
            p: 'p',
            r: 'r',
            x: 'x',
        }
    }
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.ends_with('s') {
        let seconds_str = s.trim_end_matches('s');
        seconds_str
            .parse::<u64>()
            .map(Duration::from_secs)
            .map_err(de::Error::custom)
    } else {
        Err(de::Error::custom("expected duration string like \"5s\""))
    }
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Configuration loaded successfully:")?;
        writeln!(f, "  Interval: {:?}", self.interval)?;
        writeln!(f, "  Sessions Directory: {:?}", self.sessions_dir)?;
        writeln!(f, "  Snapshot Directory: {:?}", self.snapshot_dir)?;
        writeln!(f, "  Keys:")?;
        writeln!(f, "    'v': {}", self.keys.v)?;
        writeln!(f, "    'h': {}", self.keys.h)?;
        writeln!(f, "    'p': {}", self.keys.p)?;
        writeln!(f, "    'r': {}", self.keys.r)?;
        writeln!(f, "    'x': {}", self.keys.x)?;
        Ok(())
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let app_name_str = app_name();

        let mut builder = config::Config::builder();

        // 1. Add default values.
        let default_config = AppConfig::default();
        builder = builder
            .set_default(
                "interval",
                default_config.interval.as_secs().to_string() + "s",
            )?
            .set_default(
                "sessions_dir",
                default_config.sessions_dir.to_str().unwrap_or(""),
            )?
            .set_default(
                "snapshot_dir",
                default_config.snapshot_dir.to_str().unwrap_or(""),
            )?
            .set_default("keys.v", default_config.keys.v.to_string())?
            .set_default("keys.h", default_config.keys.h.to_string())?
            .set_default("keys.p", default_config.keys.p.to_string())?
            .set_default("keys.r", default_config.keys.r.to_string())?
            .set_default("keys.x", default_config.keys.x.to_string())?;

        // 2. Add optional config file from user's config directory (overrides defaults).
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

        // 3. Add environment variables (overrides files).
        builder = builder.add_source(
            config::Environment::with_prefix(&app_name_str.to_uppercase().replace('-', "__"))
                .separator("__"),
        );

        // 4. Build and deserialize.
        builder.build()?.try_deserialize()
    }
}
