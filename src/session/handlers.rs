use std::fs;
use std::io::{self, ErrorKind};

use super::models::SessionState;
use super::utils::generate_session_filename;
use crate::app::App;

pub fn save_session_by_name(app: &App, session_filename: &str) -> io::Result<String> {
    let session_state = SessionState {
        pane_manager: app.pane_manager.clone(),
        tasks: app
            .pane_manager
            .get_all_pane_keys()
            .into_iter()
            .filter_map(|id| {
                app.tasks
                    .get(&id)
                    .map(|command| (id, command.to_serializable_state()))
            })
            .collect(),
    };

    let toml_string = toml::to_string(&session_state)
        .map_err(|e| io::Error::other(format!("Serialization error: {}", e)))?;

    let sessions_dir = &app.config.sessions_dir;
    fs::create_dir_all(sessions_dir)?;

    use std::path::Path;
    let path = Path::new(session_filename);

    let final_filename = if path.extension().is_some_and(|ext| ext == "toml") {
        session_filename.to_string()
    } else {
        format!("{}.toml", session_filename)
    };

    let session_path = sessions_dir.join(&final_filename);

    fs::write(session_path, toml_string)?;
    Ok(final_filename.trim_end_matches(".toml").to_string())
}

pub fn save_session(app: &App) -> io::Result<String> {
    let session_filename = generate_session_filename();

    save_session_by_name(app, &session_filename)
}

pub fn load_session_by_name(app: &mut App, session_filename: &str) -> io::Result<()> {
    let sessions_dir = &app.config.sessions_dir;
    let session_path = sessions_dir.join(session_filename);

    if !session_path.exists() || !session_path.is_file() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("Session file not found: {}", session_filename),
        ));
    }

    let toml_string = fs::read_to_string(session_path)?;
    let session_state: SessionState = toml::from_str(&toml_string).map_err(|e| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Deserialization error: {}", e),
        )
    })?;

    let _ = app.load_session(session_state.pane_manager, session_state.tasks);

    Ok(())
}

pub fn load_latest_session(app: &mut App) -> io::Result<String> {
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

    let latest_filename = latest_session_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "Invalid filename"))?;

    let name = latest_filename.trim_end_matches(".toml").to_string();
    load_session_by_name(app, latest_filename)?;
    Ok(name)
}

fn session_path(app_config: &crate::config::AppConfig, file_name: &str) -> std::path::PathBuf {
    app_config.sessions_dir.join(file_name)
}

pub fn delete_session(config: &crate::config::AppConfig, file_name: &str) -> io::Result<()> {
    fs::remove_file(session_path(config, file_name))
}

pub fn rename_session(
    config: &crate::config::AppConfig,
    file_name: &str,
    new_name: &str,
) -> io::Result<String> {
    let new_name = new_name.trim().trim_end_matches(".toml");
    if new_name.is_empty() || new_name.contains(['/', '\\']) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Session names can't be empty or contain slashes",
        ));
    }

    let new_file_name = format!("{new_name}.toml");
    let target = session_path(config, &new_file_name);
    if new_file_name != file_name && target.exists() {
        return Err(io::Error::new(
            ErrorKind::AlreadyExists,
            format!("A session named {new_name} already exists"),
        ));
    }

    fs::rename(session_path(config, file_name), target)?;
    Ok(new_file_name)
}
