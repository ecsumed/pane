use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::time::Duration;

use serde::Deserialize;
use serde_with::serde_as;

use super::models::PaneKeyAsString;
use crate::config::AppConfig;
use crate::pane::{PaneKey, PaneManager};
use crate::ui::DisplayType;

#[derive(Debug)]
pub struct SessionPreview {
    pub pane_manager: PaneManager,
    pub panes: Vec<PreviewPane>,
}

#[derive(Debug, PartialEq)]
pub struct PreviewPane {
    pub id: usize,
    pub exec: String,
    pub interval: Duration,
    pub display_type: DisplayType,
}

#[cfg(test)]
pub fn preview_for_tests(
    pane_manager: PaneManager,
    panes: &[(usize, &str, u64)],
) -> SessionPreview {
    SessionPreview {
        pane_manager,
        panes: panes
            .iter()
            .map(|&(id, exec, secs)| PreviewPane {
                id,
                exec: exec.to_string(),
                interval: Duration::from_secs(secs),
                display_type: DisplayType::RawText,
            })
            .collect(),
    }
}

#[serde_as]
#[derive(Deserialize)]
struct PreviewState {
    pane_manager: PaneManager,
    #[serde_as(as = "HashMap<PaneKeyAsString, _>")]
    tasks: HashMap<PaneKey, PreviewTask>,
}

#[derive(Deserialize)]
struct PreviewTask {
    exec: String,
    interval: Duration,
    display_type: DisplayType,
}

fn is_table_header(line: &str) -> bool {
    let inner = line
        .strip_prefix("[[")
        .and_then(|l| l.strip_suffix("]]"))
        .or_else(|| line.strip_prefix('[').and_then(|l| l.strip_suffix(']')));

    inner.is_some_and(|name| name.starts_with("tasks.") || name.starts_with("pane_manager"))
}

fn multiline_state(line: &str, mut open: Option<&'static str>) -> Option<&'static str> {
    let mut rest = line;
    loop {
        match open {
            Some(delimiter) => match rest.find(delimiter) {
                Some(i) => {
                    rest = &rest[i + 3..];
                    open = None;
                }
                None => return open,
            },
            None => {
                let next = ["\"\"\"", "\'\'\'"]
                    .into_iter()
                    .filter_map(|d| rest.find(d).map(|i| (i, d)))
                    .min_by_key(|&(i, _)| i);
                match next {
                    Some((i, delimiter)) => {
                        rest = &rest[i + 3..];
                        open = Some(delimiter);
                    }
                    None => return None,
                }
            }
        }
    }
}

fn strip_history(contents: &str) -> String {
    let mut kept = String::new();
    let mut in_history = false;
    let mut open_string = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        if open_string.is_none() && is_table_header(trimmed) {
            in_history = trimmed.contains(".output_history");
        }
        if !in_history {
            kept.push_str(line);
            kept.push('\n');
        }
        open_string = multiline_state(line, open_string);
    }

    kept
}

pub fn load_session_preview(config: &AppConfig, file_name: &str) -> io::Result<SessionPreview> {
    let contents = fs::read_to_string(config.sessions_dir.join(file_name))?;
    let state: PreviewState = toml::from_str(&strip_history(&contents))
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e.message().to_string()))?;

    let mut panes: Vec<PreviewPane> = state
        .tasks
        .into_iter()
        .map(|(key, task)| PreviewPane {
            id: state
                .pane_manager
                .pane_key_to_friendly_id(&key)
                .unwrap_or(0),
            exec: task.exec,
            interval: task.interval,
            display_type: task.display_type,
        })
        .collect();
    panes.sort_by_key(|p| p.id);

    Ok(SessionPreview {
        pane_manager: state.pane_manager,
        panes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::command::CommandOutput;
    use crate::session::save_session_by_name;

    #[tokio::test]
    async fn test_preview_reads_layout_and_commands_without_history() {
        let dir = std::env::temp_dir().join(format!("pane-preview-{}", std::process::id()));
        let config = AppConfig {
            sessions_dir: dir.clone(),
            ..AppConfig::default()
        };
        let mut app = App::new(config, Vec::new());

        let first = app.pane_manager.active_pane_id;
        app.set_command(first, "kubectl get pods".to_string()).await;
        app.pane_manager
            .split_pane(ratatui::layout::Direction::Horizontal);
        let second = app.pane_manager.active_pane_id;
        app.set_command(second, "date".to_string()).await;

        for task in app.tasks.values_mut() {
            task.task_handle.take().unwrap().abort();
            task.record_output(
                CommandOutput {
                    output: "[\n  [tasks]\n  [[pane_manager]]\n]\nexec = \"fake\"\n".to_string(),
                    time: chrono::Local::now().naive_local(),
                    exit_status: Some(0),
                    duration: Duration::from_millis(1),
                },
                10,
            );
        }

        save_session_by_name(&app, "preview").unwrap();
        let preview = load_session_preview(&app.config, "preview.toml");
        fs::remove_dir_all(&dir).unwrap();

        let preview = preview.unwrap();
        let execs: Vec<&str> = preview.panes.iter().map(|p| p.exec.as_str()).collect();
        assert_eq!(execs, vec!["kubectl get pods", "date"]);
        assert_eq!(preview.panes[0].id, 1);
        assert_eq!(preview.pane_manager.get_all_pane_keys().len(), 2);
    }

    #[test]
    fn test_strip_history_keeps_only_layout_and_tasks() {
        let contents = "[pane_manager]\nid_counter = 2\n[tasks.1]\nexec = \"ls\"\n[[tasks.1.output_history]]\noutput = \"\"\"\n[x]\n\"\"\"\n[tasks.1.output_history.duration]\nsecs = 1\n[tasks.1.interval]\nsecs = 5\n";
        let stripped = strip_history(contents);
        assert!(stripped.contains("exec = \"ls\""));
        assert!(stripped.contains("[tasks.1.interval]"));
        assert!(!stripped.contains("output"));
    }
}
