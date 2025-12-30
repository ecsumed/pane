use std::path::PathBuf;
use std::time::Duration;

use directories::ProjectDirs;
use serde::{Deserialize, Deserializer, Serializer, de};

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
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

pub fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}
