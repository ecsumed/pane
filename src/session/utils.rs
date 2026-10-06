use std::fs;
use std::io::{self};
use std::time::SystemTime;

use chrono::{DateTime, Local};

use crate::config::AppConfig;

pub fn generate_session_filename() -> String {
    let now: DateTime<Local> = Local::now();
    let timestamp = now.format("%Y%m%dT%H%M%S").to_string();
    format!("session-{}.toml", timestamp)
}

pub fn is_session_file(entry: &fs::DirEntry) -> bool {
    entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
        && entry
            .path()
            .extension()
            .map(|ext| ext == "toml")
            .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionEntry {
    pub file_name: String,
    pub name: String,
    pub modified: Option<SystemTime>,
    pub size: u64,
}

pub fn fetch_sessions(config: &AppConfig) -> io::Result<Vec<SessionEntry>> {
    let sessions_dir = &config.sessions_dir;

    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions: Vec<SessionEntry> = fs::read_dir(sessions_dir)?
        .filter_map(Result::ok)
        .filter(is_session_file)
        .filter_map(|entry| {
            let file_name = entry.file_name().into_string().ok()?;
            let metadata = entry.metadata().ok();
            Some(SessionEntry {
                name: file_name.trim_end_matches(".toml").to_string(),
                modified: metadata.as_ref().and_then(|m| m.modified().ok()),
                size: metadata.map_or(0, |m| m.len()),
                file_name,
            })
        })
        .collect();

    sessions.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.name.cmp(&b.name)));
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_sessions_sorted_newest_first() {
        let dir = std::env::temp_dir().join(format!("pane-sessions-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();

        let now = SystemTime::now();
        for (name, age) in [("old", 3_600), ("newest", 0), ("middle", 60)] {
            let file = fs::File::create(dir.join(format!("{name}.toml"))).unwrap();
            file.set_modified(now - Duration::from_secs(age)).unwrap();
        }
        fs::write(dir.join("notes.txt"), "").unwrap();

        let config = AppConfig {
            sessions_dir: dir.clone(),
            ..AppConfig::default()
        };
        let names: Vec<String> = fetch_sessions(&config)
            .unwrap()
            .into_iter()
            .map(|s| s.name)
            .collect();
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(names, vec!["newest", "middle", "old"]);
    }
}
