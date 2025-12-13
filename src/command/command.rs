use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::ui::DisplayType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandControl {
    Resume,
    Stop,
    Pause,
    IntervalIncrease,
    IntervalDecrease,
    IntervalSet(Duration),
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CommandState {
    Running,
    Paused,
    Stopped,
}

impl fmt::Display for CommandState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandState::Running => write!(f, "RUNNING"),
            CommandState::Paused => write!(f, "PAUSED"),
            CommandState::Stopped => write!(f, "STOPPED"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandSerializableState {
    pub exec: String,
    pub interval: Duration,
    pub output_history: Vec<String>,
    pub last_output: String,
    pub state: CommandState,
    pub display_type: DisplayType,
}

#[derive(Debug)]
pub struct Command {
    pub exec: String,
    pub interval: Duration,
    pub output_history: Vec<String>,
    pub last_output: String,
    pub state: CommandState,
    pub display_type: DisplayType,
    pub task_handle: Option<JoinHandle<()>>,
    pub control_tx: mpsc::Sender<CommandControl>,
}

impl Command {
    pub fn to_serializable_state(&self) -> CommandSerializableState {
        CommandSerializableState {
            exec: self.exec.clone(),
            interval: self.interval,
            output_history: self.output_history.clone(),
            last_output: self.last_output.clone(),
            state: self.state,
            display_type: self.display_type,
        }
    }
}
