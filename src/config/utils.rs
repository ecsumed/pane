use serde::{de, Deserializer, Deserialize};
use std::path::PathBuf;
use std::time::Duration;

use directories::ProjectDirs;


pub fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

pub fn get_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub fn default_sessions_dir_path(proj_dirs: &Option<ProjectDirs>) -> PathBuf {
    #[cfg(target_os = "macos")]
    if let Some(home_dir) = get_home_dir() {
        return home_dir.join(".config").join(app_name()).join("sessions");
    }

    if let Some(dirs) = proj_dirs {
        dirs.data_dir().join("sessions")
    } else {
        PathBuf::from("./data/sessions")
    }
}

pub fn default_snapshot_dir_path(proj_dirs: &Option<ProjectDirs>) -> PathBuf {
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

pub fn default_logging_dir_path(proj_dirs: &Option<ProjectDirs>) -> PathBuf {
    #[cfg(target_os = "macos")]
    if let Some(home_dir) = get_home_dir() {
        return home_dir.join(".config").join(app_name()).join("logs");
    }

    if let Some(dirs) = proj_dirs {
        dirs.data_dir().join("logs")
    } else {
        PathBuf::from("./data/logs")
    }
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
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