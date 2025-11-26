use crate::app::App;
use crate::command::{Command, CommandSerializableState, CommandState};
use crate::config::AppConfig;
use crate::pane_manager::PaneManager;
use chrono::{DateTime, Local};
use serde::{de, Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::logging::{debug, info, warn};

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct SessionState {
    pub pane_manager: PaneManager,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub tasks: HashMap<usize, CommandSerializableState>,
}

fn generate_session_filename() -> String {
    let now: DateTime<Local> = Local::now();
    let timestamp = now.format("%Y%m%dT%H%M%S").to_string();
    format!("session-{}.toml", timestamp)
}

pub fn save_session(app: &App) -> io::Result<()> {
    let session_state = SessionState {
        pane_manager: app.pane_manager.clone(),
        tasks: app
            .tasks
            .iter()
            .map(|(&id, command)| (id, command.to_serializable_state()))
            .collect(),
    };

    let toml_string = toml::to_string(&session_state)
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Serialization error: {}", e)))?;

    let sessions_dir = &app.config.sessions_dir;
    fs::create_dir_all(sessions_dir)?;

    let session_filename = generate_session_filename();
    let session_path = sessions_dir.join(session_filename);

    fs::write(session_path, toml_string)?;
    Ok(())
}

pub fn load_session(app: &mut App) -> io::Result<()> {
    let sessions_dir = &app.config.sessions_dir;

    if !sessions_dir.exists() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            "Sessions directory not found",
        ));
    }

    let latest_session_path = fs::read_dir(sessions_dir)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().map(|ext| ext == "toml").unwrap_or(false) {
                Some(path)
            } else {
                None
            }
        })
        .max()
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "No session files found"))?;

    let toml_string = fs::read_to_string(latest_session_path)?;
    let session_state: SessionState = toml::from_str(&toml_string).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Deserialization error: {}", e),
        )
    })?;

    let _ = app.load_session(session_state.pane_manager, session_state.tasks);

    Ok(())
}
