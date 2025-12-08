use std::env;
use std::fs;
use std::path::PathBuf;
use std::io;
use crate::logging::{debug, info, warn};

#[derive(Debug)]
pub struct HistoryManager {
    commands: Vec<String>,
}

impl HistoryManager {
    pub fn new() -> Self {
        let commands = Self::load_history_file().unwrap();

        debug!("history length: {}", commands.len());
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

        let path = PathBuf::from(history_file_path);
        
        if !path.exists() {
            warn!("History file not found at {:?}", path);
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(path)?;
        let commands: Vec<String> = contents
            .lines()
            .filter(|&line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|line| line.trim().to_string())
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
