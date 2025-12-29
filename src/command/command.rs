use std::collections::VecDeque;
use std::fmt;
use std::time::Duration;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::command::serialization::naivedatetime_format;
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
    Idle,
    Paused,
    Executing,
    Stopped,
}

impl fmt::Display for CommandState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandState::Idle => write!(f, "IDLE"),
            CommandState::Paused => write!(f, "PAUSED"),
            CommandState::Executing => write!(f, "EXECUTING"),
            CommandState::Stopped => write!(f, "STOPPED"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandSerializableState {
    pub exec: String,
    pub interval: Duration,
    pub output_history: VecDeque<CommandOutput>,
    pub state: CommandState,
    pub display_type: DisplayType,
}

#[derive(Debug)]
pub struct Command {
    pub exec: String,
    pub interval: Duration,
    pub output_history: VecDeque<CommandOutput>,
    pub state: CommandState,
    pub display_type: DisplayType,
    pub task_handle: Option<JoinHandle<()>>,
    pub control_tx: mpsc::Sender<CommandControl>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandOutput {
    pub output: String,
    #[serde(with = "naivedatetime_format")]
    pub time: NaiveDateTime,
    pub exit_status: Option<i32>,
    pub duration: Duration,
}

#[derive(Debug)]
pub enum CommandEvent {
    Started,
    Output(CommandOutput),
}

impl Command {
    pub fn to_serializable_state(&self) -> CommandSerializableState {
        CommandSerializableState {
            exec: self.exec.clone(),
            interval: self.interval,
            output_history: self.output_history.clone(),
            state: self.state,
            display_type: self.display_type,
        }
    }
}
