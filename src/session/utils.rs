use std::fs;
use std::io::{self};

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

pub fn fetch_session_filenames(config: &AppConfig) -> io::Result<Vec<String>> {
    let sessions_dir = &config.sessions_dir;

    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let filenames: Vec<String> = fs::read_dir(sessions_dir)?
        .filter_map(Result::ok)
        .filter(is_session_file)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    Ok(filenames)
}
