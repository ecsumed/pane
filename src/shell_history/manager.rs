use std::path::PathBuf;
use std::{env, fs, io};

use crate::logging::{debug, trace, warn};

#[derive(Debug, Clone)]
pub struct ShellHistoryManager {
    commands: Vec<String>,
}

impl ShellHistoryManager {
    pub fn new() -> Self {
        let commands = match Self::load_history_file() {
            Ok(cmds) => cmds,
            Err(e) => {
                warn!("Warning: Could not load history due to invalid data: {}", e);
                Vec::new()
            }
        };

        debug!("history length: {}", commands.len());
        trace!("{:?}", commands);
        Self { commands }
    }

    fn load_history_file() -> io::Result<Vec<String>> {
        let shell = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
        let home_dir = env::var("HOME").expect("$HOME environment variable not set");

        let history_file_path = if shell.contains("zsh") {
            format!("{home_dir}/.zsh_history")
        } else {
            format!("{home_dir}/.bash_history")
        };

        debug!("Shell history file path: {}", history_file_path);

        let path = PathBuf::from(history_file_path);

        if !path.exists() {
            warn!("History file not found at {:?}", path);
            return Ok(Vec::new());
        }

        let bytes = fs::read(path)?;
        let contents = String::from_utf8_lossy(&bytes);

        let commands: Vec<String> = contents
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#')
            })
            .map(|line| {
                let trimmed = line.trim();
                match trimmed.split_once(';') {
                    Some((_metadata, command)) => command.trim().to_string(),
                    None => trimmed.to_string(),
                }
            })
            .collect();

        Ok(commands)
    }

    pub fn filter(&self, input: &str) -> Vec<String> {
        if input.is_empty() {
            return Vec::new();
        }
        self.commands
            .iter()
            .rev()
            .filter(|cmd| cmd.starts_with(input))
            .take(10)
            .cloned()
            .collect()
    }
}
