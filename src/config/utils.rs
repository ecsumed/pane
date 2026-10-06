use std::path::PathBuf;
use std::time::Duration;

use directories::ProjectDirs;
use serde::{Deserialize, Deserializer, Serializer};

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

pub fn parse_duration(text: &str) -> Result<Duration, String> {
    let text = text.trim();
    let duration = match text.parse::<f64>() {
        Ok(secs) if secs.is_finite() && secs >= 0.0 => Duration::from_secs_f64(secs),
        Ok(_) => return Err(format!("invalid duration {text:?}")),
        Err(_) => humantime::parse_duration(text)
            .map_err(|e| format!("invalid duration {text:?}: {e}"))?,
    };

    if duration.is_zero() {
        return Err(format!("duration {text:?} must be greater than zero"));
    }
    Ok(duration)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DurationValue {
    Whole(u64),
    Fraction(f64),
    Text(String),
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let text = match DurationValue::deserialize(deserializer)? {
        DurationValue::Whole(secs) => secs.to_string(),
        DurationValue::Fraction(secs) => secs.to_string(),
        DurationValue::Text(text) => text,
    };
    parse_duration(&text).map_err(serde::de::Error::custom)
}

pub fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&humantime::format_duration(*duration).to_string())
}

#[cfg(test)]
mod tests {
    use figment::providers::{Format, Serialized, Toml};
    use figment::Figment;

    use super::*;
    use crate::config::AppConfig;

    fn load(toml: &str) -> Result<AppConfig, figment::Error> {
        Figment::new()
            .merge(Serialized::defaults(AppConfig::default()))
            .merge(Toml::string(toml))
            .extract()
    }

    #[test]
    fn test_parse_duration_formats() {
        assert_eq!(parse_duration("5"), Ok(Duration::from_secs(5)));
        assert_eq!(parse_duration("0.5"), Ok(Duration::from_millis(500)));
        assert_eq!(parse_duration("500ms"), Ok(Duration::from_millis(500)));
        assert_eq!(parse_duration("1m30s"), Ok(Duration::from_secs(90)));
        assert!(parse_duration("0").is_err());
        assert!(parse_duration("-1").is_err());
        assert!(parse_duration("soon").is_err());
    }

    #[test]
    fn test_config_accepts_numbers_and_strings() {
        let config = load("interval = 3\ntimeout = 2.5").unwrap();
        assert_eq!(config.interval, Duration::from_secs(3));
        assert_eq!(config.timeout, Duration::from_millis(2_500));

        let config = load("interval = \"500ms\"\ntimeout = \"2m\"").unwrap();
        assert_eq!(config.interval, Duration::from_millis(500));
        assert_eq!(config.timeout, Duration::from_secs(120));

        assert!(load("interval = 0").is_err());
    }

    #[test]
    fn test_defaults_round_trip() {
        let config = load("").unwrap();
        assert_eq!(config.interval, AppConfig::default().interval);
        assert_eq!(config.timeout, AppConfig::default().timeout);
    }
}
