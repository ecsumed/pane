use std::collections::HashSet;
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

    #[cfg(test)]
    pub fn from_commands(commands: Vec<String>) -> Self {
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

        Ok(Self::parse_history(&contents))
    }

    fn parse_history(contents: &str) -> Vec<String> {
        contents
            .lines()
            .map(|line| Self::strip_zsh_metadata(line.trim()).trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(String::from)
            .collect()
    }

    fn strip_zsh_metadata(line: &str) -> &str {
        let Some((meta, command)) = line
            .strip_prefix(": ")
            .and_then(|rest| rest.split_once(';'))
        else {
            return line;
        };

        let is_metadata = meta
            .split(':')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));

        if is_metadata {
            command
        } else {
            line
        }
    }

    pub fn filter(&self, input: &str) -> Vec<String> {
        if input.is_empty() {
            return Vec::new();
        }
        let mut seen = HashSet::new();
        self.commands
            .iter()
            .rev()
            .filter(|cmd| cmd.starts_with(input) && seen.insert(cmd.as_str()))
            .take(10)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_zsh_extended_history() {
        let contents = ": 1700000000:0;kubectl get pods\n: 1700000001:12;echo a; echo b\n";
        assert_eq!(
            ShellHistoryManager::parse_history(contents),
            vec!["kubectl get pods", "echo a; echo b"]
        );
    }

    #[test]
    fn test_parse_bash_history_keeps_semicolons() {
        let contents = "#1700000000\nfor i in 1 2; do echo $i; done\nls -la\n";
        assert_eq!(
            ShellHistoryManager::parse_history(contents),
            vec!["for i in 1 2; do echo $i; done", "ls -la"]
        );
    }

    #[test]
    fn test_filter_dedupes_most_recent_first() {
        let history = ShellHistoryManager::from_commands(
            ["kubectl get pods", "kubectl top pod", "kubectl get pods"]
                .map(String::from)
                .to_vec(),
        );
        assert_eq!(
            history.filter("kubectl"),
            vec!["kubectl get pods", "kubectl top pod"]
        );
    }
}
